// Kiren Axios Compatibility Layer
// Drop-in replacement for axios

function createInstance(defaultConfig = {}) {
  const instance = {
    defaults: {
      headers: {
        common: {},
        get: {},
        post: { 'Content-Type': 'application/json' },
        put: { 'Content-Type': 'application/json' },
        delete: {}
      },
      ...defaultConfig
    },

    // Main request method
    request: function(config) {
      return makeRequest({ ...this.defaults, ...config });
    },

    get: function(url, config = {}) {
      return this.request({ ...config, method: 'GET', url });
    },

    post: function(url, data, config = {}) {
      return this.request({ ...config, method: 'POST', url, data });
    },

    put: function(url, data, config = {}) {
      return this.request({ ...config, method: 'PUT', url, data });
    },

    delete: function(url, config = {}) {
      return this.request({ ...config, method: 'DELETE', url });
    },

    patch: function(url, data, config = {}) {
      return this.request({ ...config, method: 'PATCH', url, data });
    },

    // Create new instance with merged config
    create: function(config = {}) {
      return createInstance({ ...this.defaults, ...config });
    }
  };

  return instance;
}

function makeRequest(config) {
  const {
    method = 'GET',
    url,
    baseURL = '',
    headers = {},
    data,
    params,
    auth,
    timeout
  } = config;

  // Build full URL
  let fullUrl = baseURL + url;

  // Add query params
  if (params) {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      searchParams.append(key, value);
    });
    const queryString = searchParams.toString();
    if (queryString) {
      fullUrl += (fullUrl.includes('?') ? '&' : '?') + queryString;
    }
  }

  // Build headers
  const requestHeaders = { ...headers };

  // Add auth header if provided
  if (auth && auth.username && auth.password) {
    const credentials = Buffer.from(`${auth.username}:${auth.password}`).toString();
    // Simple base64 encoding for basic auth
    const encoded = btoa(`${auth.username}:${auth.password}`);
    requestHeaders['Authorization'] = `Basic ${encoded}`;
  }

  // Build fetch options
  const fetchOptions = {
    method,
    headers: requestHeaders
  };

  // Add body for POST/PUT/PATCH
  if (data && ['POST', 'PUT', 'PATCH'].includes(method.toUpperCase())) {
    if (typeof data === 'object') {
      fetchOptions.body = JSON.stringify(data);
      if (!requestHeaders['Content-Type']) {
        requestHeaders['Content-Type'] = 'application/json';
      }
    } else {
      fetchOptions.body = data;
    }
  }

  // Make request
  try {
    const response = fetch(fullUrl, fetchOptions);

    // Build axios-compatible response
    const axiosResponse = {
      data: null,
      status: response.status,
      statusText: response.statusText || '',
      headers: response.headers || {},
      config: config,
      request: null
    };

    // Parse response body
    const contentType = response.headers && response.headers['content-type'];
    if (contentType && contentType.includes('application/json')) {
      try {
        axiosResponse.data = response.json();
      } catch (e) {
        axiosResponse.data = response.text();
      }
    } else {
      axiosResponse.data = response.text();
    }

    // Check for error status
    if (response.status >= 400) {
      const error = new Error(`Request failed with status ${response.status}`);
      error.response = axiosResponse;
      error.config = config;
      error.isAxiosError = true;
      throw error;
    }

    return axiosResponse;
  } catch (e) {
    if (e.isAxiosError) throw e;

    const error = new Error(e.message || 'Network Error');
    error.config = config;
    error.isAxiosError = true;
    throw error;
  }
}

// Base64 encoding for basic auth (browser-compatible btoa)
function btoa(str) {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
  let output = '';

  for (let i = 0; i < str.length; i += 3) {
    const char1 = str.charCodeAt(i);
    const char2 = str.charCodeAt(i + 1);
    const char3 = str.charCodeAt(i + 2);

    const enc1 = char1 >> 2;
    const enc2 = ((char1 & 3) << 4) | (char2 >> 4);
    const enc3 = ((char2 & 15) << 2) | (char3 >> 6);
    const enc4 = char3 & 63;

    if (isNaN(char2)) {
      output += chars.charAt(enc1) + chars.charAt(enc2) + '==';
    } else if (isNaN(char3)) {
      output += chars.charAt(enc1) + chars.charAt(enc2) + chars.charAt(enc3) + '=';
    } else {
      output += chars.charAt(enc1) + chars.charAt(enc2) + chars.charAt(enc3) + chars.charAt(enc4);
    }
  }

  return output;
}

// Create default instance
const axios = createInstance();
axios.create = (config) => createInstance(config);

// CancelToken (stub for compatibility)
axios.CancelToken = {
  source: () => ({
    token: {},
    cancel: () => {}
  })
};

axios.isCancel = () => false;
axios.isAxiosError = (e) => e && e.isAxiosError === true;

module.exports = axios;
