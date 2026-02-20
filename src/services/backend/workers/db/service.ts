import { createWorkerApi } from '../factoryMethods';
import { CozoDbWorker } from './worker';

const workerUrl = new URL('./worker.ts', import.meta.url);

export const { workerApiProxy: cozoDbWorkerInstance } = createWorkerApi<CozoDbWorker>(
  workerUrl,
  'cyb~cozodb'
);
