import { createWorkerApi } from '../factoryMethods';
import { BackgroundWorker } from './worker';

const workerUrl = new URL('./worker.ts', import.meta.url);

export const { workerApiProxy: backgroundWorkerInstance } = createWorkerApi<BackgroundWorker>(
  workerUrl,
  'cyb~backend'
);

// export const backendApi;
