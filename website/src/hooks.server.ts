import { config } from 'dotenv';
import { resolve } from 'path';

// Load .env file from the website directory
config({ path: resolve(process.cwd(), '.env') });
