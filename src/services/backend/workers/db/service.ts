import { CozoDbWorker } from './worker';
import { createWorkerApi } from '../factoryMethods';

const workerUrl = new URL('./worker.ts', import.meta.url);

export const { workerApiProxy: cozoDbWorkerInstance } =
  createWorkerApi<CozoDbWorker>(workerUrl, 'cyb~cozodb');
