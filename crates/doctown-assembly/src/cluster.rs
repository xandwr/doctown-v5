//! Vector clustering using k-means algorithm.

use ndarray::{Array2, Axis};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Result of clustering operation.
#[derive(Debug, Clone)]
pub struct ClusterResult {
    /// Cluster assignment for each vector (0-indexed).
    pub assignments: Vec<usize>,
    /// Centroid positions for each cluster.
    pub centroids: Array2<f32>,
    /// Number of members in each cluster.
    pub cluster_sizes: Vec<usize>,
}

/// Clusterer for grouping similar vectors.
pub struct Clusterer {
    /// Number of clusters to create.
    k: usize,
    /// Maximum iterations for k-means.
    max_iter: usize,
    /// Convergence tolerance.
    tolerance: f32,
}

impl Clusterer {
    /// Create a new clusterer with the specified number of clusters.
    pub fn new(k: usize) -> Self {
        Self {
            k,
            max_iter: 300,
            tolerance: 1e-4,
        }
    }

    /// Create a clusterer with custom parameters.
    pub fn with_params(k: usize, max_iter: usize, tolerance: f32) -> Self {
        Self {
            k,
            max_iter,
            tolerance,
        }
    }

    /// Cluster vectors using k-means.
    /// Returns cluster assignments for each vector.
    pub fn cluster(&self, vectors: &Array2<f32>) -> Result<ClusterResult, String> {
        if vectors.nrows() == 0 {
            return Err("Cannot cluster empty vector set".to_string());
        }

        if self.k > vectors.nrows() {
            return Err(format!(
                "Cannot create {} clusters from {} vectors",
                self.k,
                vectors.nrows()
            ));
        }

        // Initialize centroids randomly (k-means++)
        let mut centroids = self.initialize_centroids(vectors)?;
        let mut assignments = vec![0; vectors.nrows()];
        
        // K-means iterations
        for _iter in 0..self.max_iter {
            // Assignment step: assign each point to nearest centroid
            let mut changed = false;
            for (i, vector) in vectors.axis_iter(Axis(0)).enumerate() {
                let new_cluster = self.nearest_centroid(&vector, &centroids);
                if assignments[i] != new_cluster {
                    assignments[i] = new_cluster;
                    changed = true;
                }
            }

            // If no assignments changed, we've converged
            if !changed {
                break;
            }

            // Update step: recompute centroids
            let old_centroids = centroids.clone();
            centroids = self.compute_centroids(vectors, &assignments)?;

            // Check convergence by centroid movement
            let max_movement = self.max_centroid_movement(&old_centroids, &centroids);
            if max_movement < self.tolerance {
                break;
            }
        }

        // Compute cluster sizes
        let mut cluster_sizes = vec![0; self.k];
        for &assignment in &assignments {
            cluster_sizes[assignment] += 1;
        }

        Ok(ClusterResult {
            assignments,
            centroids,
            cluster_sizes,
        })
    }

    /// Initialize centroids using k-means++ algorithm.
    fn initialize_centroids(&self, vectors: &Array2<f32>) -> Result<Array2<f32>, String> {
        let mut rng = thread_rng();
        let n = vectors.nrows();
        let dim = vectors.ncols();
        
        // Choose first centroid randomly
        let indices: Vec<usize> = (0..n).collect();
        let first_idx = *indices.choose(&mut rng)
            .ok_or("No vectors to choose from")?;
        
        let mut centroids = Array2::zeros((self.k, dim));
        centroids.row_mut(0).assign(&vectors.row(first_idx));
        
        // Choose remaining centroids with probability proportional to distance squared
        for k in 1..self.k {
            let mut distances = vec![0.0f32; n];
            for i in 0..n {
                let vector = vectors.row(i);
                let mut min_dist = f32::MAX;
                for j in 0..k {
                    let centroid = centroids.row(j);
                    let dist = self.euclidean_distance(&vector, &centroid);
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }
                distances[i] = min_dist * min_dist;
            }
            
            // Select next centroid with weighted probability
            let total: f32 = distances.iter().sum();
            if total > 0.0 {
                let mut cumsum = vec![0.0f32; n];
                cumsum[0] = distances[0] / total;
                for i in 1..n {
                    cumsum[i] = cumsum[i - 1] + distances[i] / total;
                }
                
                let rand_val: f32 = rand::random();
                let idx = cumsum.iter().position(|&x| x >= rand_val).unwrap_or(n - 1);
                centroids.row_mut(k).assign(&vectors.row(idx));
            } else {
                // Fallback to random selection
                let idx = *indices.choose(&mut rng)
                    .ok_or("No vectors to choose from")?;
                centroids.row_mut(k).assign(&vectors.row(idx));
            }
        }
        
        Ok(centroids)
    }

    /// Find the nearest centroid to a vector.
    fn nearest_centroid(&self, vector: &ndarray::ArrayView1<f32>, centroids: &Array2<f32>) -> usize {
        let mut min_dist = f32::MAX;
        let mut nearest = 0;
        
        for (i, centroid) in centroids.axis_iter(Axis(0)).enumerate() {
            let dist = self.euclidean_distance(vector, &centroid);
            if dist < min_dist {
                min_dist = dist;
                nearest = i;
            }
        }
        
        nearest
    }

    /// Compute new centroids based on current assignments.
    fn compute_centroids(&self, vectors: &Array2<f32>, assignments: &[usize]) -> Result<Array2<f32>, String> {
        let dim = vectors.ncols();
        let mut centroids = Array2::zeros((self.k, dim));
        let mut counts = vec![0; self.k];
        
        // Sum vectors in each cluster
        for (i, vector) in vectors.axis_iter(Axis(0)).enumerate() {
            let cluster = assignments[i];
            centroids.row_mut(cluster).scaled_add(1.0, &vector);
            counts[cluster] += 1;
        }
        
        // Divide by counts to get mean
        for k in 0..self.k {
            if counts[k] > 0 {
                centroids.row_mut(k).mapv_inplace(|x| x / counts[k] as f32);
            }
        }
        
        Ok(centroids)
    }

    /// Compute Euclidean distance between two vectors.
    fn euclidean_distance(&self, a: &ndarray::ArrayView1<f32>, b: &ndarray::ArrayView1<f32>) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Compute maximum movement of centroids.
    fn max_centroid_movement(&self, old: &Array2<f32>, new: &Array2<f32>) -> f32 {
        let mut max_movement = 0.0f32;
        for i in 0..self.k {
            let movement = self.euclidean_distance(&old.row(i), &new.row(i));
            if movement > max_movement {
                max_movement = movement;
            }
        }
        max_movement
    }

    /// Compute optimal cluster count using heuristic (sqrt(n/2)).
    pub fn optimal_k(n: usize) -> usize {
        ((n as f64 / 2.0).sqrt().ceil() as usize).max(2)
    }

    /// Get the number of clusters.
    pub fn k(&self) -> usize {
        self.k
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_optimal_k() {
        assert_eq!(Clusterer::optimal_k(100), 8);
        assert_eq!(Clusterer::optimal_k(10), 3);
        assert_eq!(Clusterer::optimal_k(2), 2);
    }

    #[test]
    fn test_clustering_simple() {
        // Create simple synthetic data: two clear clusters
        let vectors = arr2(&[
            [0.0, 0.0],
            [1.0, 1.0],
            [0.5, 0.5],
            [10.0, 10.0],
            [11.0, 11.0],
            [10.5, 10.5],
        ]);

        let clusterer = Clusterer::new(2);
        let result = clusterer.cluster(&vectors).unwrap();

        // Check we got 2 clusters
        assert_eq!(result.centroids.nrows(), 2);
        assert_eq!(result.assignments.len(), 6);
        assert_eq!(result.cluster_sizes.len(), 2);

        // Check that similar points are in same cluster
        assert_eq!(result.assignments[0], result.assignments[1]);
        assert_eq!(result.assignments[0], result.assignments[2]);
        assert_eq!(result.assignments[3], result.assignments[4]);
        assert_eq!(result.assignments[3], result.assignments[5]);

        // Check that different clusters are different
        assert_ne!(result.assignments[0], result.assignments[3]);

        // Check cluster sizes sum to total
        let total_size: usize = result.cluster_sizes.iter().sum();
        assert_eq!(total_size, 6);
    }

    #[test]
    fn test_clustering_correct_number() {
        // Test with k=3
        let vectors = arr2(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [5.0, 5.0],
            [6.0, 5.0],
            [10.0, 10.0],
            [11.0, 10.0],
        ]);

        let clusterer = Clusterer::new(3);
        let result = clusterer.cluster(&vectors).unwrap();

        assert_eq!(result.centroids.nrows(), 3);
        assert_eq!(result.cluster_sizes.len(), 3);

        // All clusters should have at least one member
        for &size in &result.cluster_sizes {
            assert!(size > 0, "Empty cluster detected");
        }
    }

    #[test]
    fn test_clustering_empty_vectors() {
        let vectors = Array2::<f32>::zeros((0, 10));
        let clusterer = Clusterer::new(2);
        let result = clusterer.cluster(&vectors);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_clustering_too_many_clusters() {
        let vectors = arr2(&[
            [0.0, 0.0],
            [1.0, 1.0],
        ]);

        let clusterer = Clusterer::new(5);
        let result = clusterer.cluster(&vectors);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot create 5 clusters"));
    }

    #[test]
    fn test_clustering_single_cluster() {
        let vectors = arr2(&[
            [0.0, 0.0],
            [1.0, 1.0],
            [0.5, 0.5],
        ]);

        let clusterer = Clusterer::new(1);
        let result = clusterer.cluster(&vectors).unwrap();

        // All points should be in cluster 0
        assert_eq!(result.assignments, vec![0, 0, 0]);
        assert_eq!(result.cluster_sizes, vec![3]);
    }

    #[test]
    fn test_euclidean_distance() {
        let clusterer = Clusterer::new(2);
        let a = arr2(&[[0.0, 0.0]]);
        let b = arr2(&[[3.0, 4.0]]);

        let dist = clusterer.euclidean_distance(&a.row(0), &b.row(0));
        assert!((dist - 5.0).abs() < 1e-6);
    }
}
