class WsClient {
    constructor() {
        this.socket = null;
        this.listeners = new Map();
        this.reconnectTimer = null;
        this.reconnectInterval = 3000;
        this.isConnected = false;
    }

    connect() {
        if (this.socket && (this.socket.readyState === WebSocket.OPEN || this.socket.readyState === WebSocket.CONNECTING)) {
            return;
        }

        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;

        try {
            this.socket = new WebSocket(wsUrl);

            this.socket.onopen = () => {
                this.isConnected = true;
                this.updateIndicator(true);
                if (this.reconnectTimer) {
                    clearTimeout(this.reconnectTimer);
                    this.reconnectTimer = null;
                }
            };

            this.socket.onmessage = (event) => {
                const message = event.data;
                this.dispatch(message);
            };

            this.socket.onclose = () => {
                this.isConnected = false;
                this.updateIndicator(false);
                this.scheduleReconnect();
            };

            this.socket.onerror = () => {
                this.isConnected = false;
                this.updateIndicator(false);
                this.socket.close();
            };
        } catch (_) {
            this.updateIndicator(false);
            this.scheduleReconnect();
        }
    }

    scheduleReconnect() {
        if (!this.reconnectTimer) {
            this.reconnectTimer = setTimeout(() => {
                this.reconnectTimer = null;
                this.connect();
            }, this.reconnectInterval);
        }
    }

    updateIndicator(connected) {
        const dot = document.querySelector('.status-dot');
        const text = document.getElementById('ws-status-text');

        if (dot) {
            dot.style.background = connected ? 'var(--green-status)' : 'var(--red, #ef4444)';
            dot.style.boxShadow = connected ? '0 0 8px var(--green-status)' : '0 0 8px var(--red, #ef4444)';
        }
        if (text) {
            text.textContent = connected ? 'CONNECTÉ' : 'DÉCONNECTÉ';
            text.style.color = connected ? 'var(--text-secondary)' : 'var(--red, #ef4444)';
        }
    }

    on(eventType, callback) {
        if (!this.listeners.has(eventType)) {
            this.listeners.set(eventType, []);
        }
        this.listeners.get(eventType).push(callback);
    }

    off(eventType, callback) {
        if (!this.listeners.has(eventType)) return;
        const list = this.listeners.get(eventType).filter(cb => cb !== callback);
        this.listeners.set(eventType, list);
    }

    dispatch(eventType) {
        if (this.listeners.has(eventType)) {
            this.listeners.get(eventType).forEach(cb => {
                try {
                    cb();
                } catch (err) {
                    console.error(`Erreur callback WS pour ${eventType}:`, err);
                }
            });
        }
    }
}

export const wsClient = new WsClient();
