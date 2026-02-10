import { createWebSocketStore } from './ws_store.svelte';

const processSocket = createWebSocketStore<Record<string, unknown>>('ws/p', {
	initialData: {}
});

/**
 * Reactive store for process data.
 * Access: pdata.data, pdata.status
 */
export const pdata = processSocket.store;

/** Open the process WebSocket connection */
export const open_ws = processSocket.open;

/** Close the process WebSocket connection */
export const close_ws = processSocket.close;
