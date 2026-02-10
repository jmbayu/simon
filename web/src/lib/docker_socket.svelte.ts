import { createWebSocketStore } from './ws_store.svelte';
import type { DockerInfo } from './types';

const dockerSocket = createWebSocketStore<DockerInfo>('ws/d', {
	initialData: {} as DockerInfo,
	skipReconnectWhenNull: true
});

/**
 * Reactive store for Docker container data.
 * Access: ddata.data, ddata.status
 */
export const ddata = dockerSocket.store;

/** Open the Docker WebSocket connection */
export const open_ws = dockerSocket.open;

/** Close the Docker WebSocket connection */
export const close_ws = dockerSocket.close;
