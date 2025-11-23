<script>
  let repoUrl = '';
  let job = null;
  let status = null;
  let polling = null;

  async function startJob() {
    status = 'Starting job...';
    const response = await fetch('/api/create-job', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ repoUrl }),
    });
    job = await response.json();
    status = `Job ${job.id} created.`;

    polling = setInterval(async () => {
      const response = await fetch(`/api/job/${job.id}`);
      const result = await response.json();
      if (result.status === 'COMPLETED') {
        clearInterval(polling);
        status = `Job completed. Result: ${JSON.stringify(result.output)}`;
      } else if (result.status === 'FAILED') {
        clearInterval(polling);
        status = `Job failed. Result: ${JSON.stringify(result.output)}`;
      } else {
        status = `Job status: ${result.status}`;
      }
    }, 2000);
  }

  
</script>

<h1>DocTown v5 Builder</h1>

<input type="text" bind:value={repoUrl} placeholder="Enter public GitHub URL" />
<button on:click={startJob}>Generate</button>

{#if status}
  <h2>Job Status</h2>
  <p>{status}</p>
{/if}
