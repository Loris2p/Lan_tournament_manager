/**
 * Client HTTP centralisé (api.js)
 * Gère les requêtes vers l'API Axum avec cookies HttpOnly,
 * parsing JSON automatique et formatage standardisé des erreurs.
 */

export class ApiError extends Error {
    constructor(status, message) {
        super(message);
        this.status = status;
        this.name = 'ApiError';
    }
}

async function request(url, options = {}) {
    const defaultHeaders = {};

    // Ne pas définir Content-Type si le body est un FormData (le navigateur gère le multipart boundary)
    const isFormData = options.body instanceof FormData;
    if (!isFormData && options.body && typeof options.body === 'object') {
        options.body = JSON.stringify(options.body);
        defaultHeaders['Content-Type'] = 'application/json';
    }

    const config = {
        ...options,
        credentials: 'same-origin', // Transit automatique des cookies de session JWT
        headers: {
            ...defaultHeaders,
            ...(options.headers || {})
        }
    };

    let response;
    try {
        response = await fetch(url, config);
    } catch (networkErr) {
        throw new ApiError(0, "Erreur réseau : impossible de joindre le serveur local.");
    }

    let data = null;
    const contentType = response.headers.get('content-type');
    if (contentType && contentType.includes('application/json')) {
        try {
            data = await response.json();
        } catch (_) {
            data = null;
        }
    } else {
        data = await response.text();
    }

    if (!response.ok) {
        const errorMsg = (data && data.error) ? data.error : (typeof data === 'string' && data.length > 0 ? data : `Erreur HTTP ${response.status}`);
        throw new ApiError(response.status, errorMsg);
    }

    return data;
}

export const api = {
    get: (url, options) => request(url, { ...options, method: 'GET' }),
    post: (url, body, options) => request(url, { ...options, method: 'POST', body }),
    patch: (url, body, options) => request(url, { ...options, method: 'PATCH', body }),
    delete: (url, options) => request(url, { ...options, method: 'DELETE' }),
    postForm: (url, formData, options) => request(url, { ...options, method: 'POST', body: formData })
};
