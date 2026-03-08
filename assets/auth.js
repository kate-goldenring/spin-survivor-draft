// Shared authentication utilities for Survivor Draft application
const SurvivorAuth = {
    /**
     * Authenticate with username and password
     * @param {string} username
     * @param {string} password
     * @returns {Promise<{success: boolean, token?: string, error?: string}>}
     */
    async login(username, password) {
        try {
            // Generate SHA-256 hash of username:password
            const encoder = new TextEncoder();
            const credentials = `${username}:${password}`;
            const hashBuffer = await crypto.subtle.digest('SHA-256', encoder.encode(credentials));

            // Convert hash to hex string
            const hashArray = Array.from(new Uint8Array(hashBuffer));
            const hashHex = hashArray
                .map(b => b.toString(16).padStart(2, '0'))
                .join('');

            // Create the Bearer token
            const authToken = `Bearer ${hashHex}`;

            // Validate the token with the backend
            const response = await fetch('/api/validate', {
                method: 'GET',
                headers: {
                    'Authorization': authToken
                }
            });

            if (response.ok) {
                // Save token for future sessions
                localStorage.setItem('survivorAuthToken', authToken);
                return { success: true, token: authToken };
            } else {
                return { success: false, error: 'Invalid username or password' };
            }
        } catch (error) {
            console.error('Login error:', error);
            return { success: false, error: 'Login failed. Please try again.' };
        }
    },

    /**
     * Validate an existing token
     * @param {string} token
     * @returns {Promise<boolean>}
     */
    async validate(token) {
        try {
            const response = await fetch('/api/validate', {
                method: 'GET',
                headers: {
                    'Authorization': token
                }
            });

            if (response.ok) {
                return true;
            } else {
                // Clear invalid token
                this.logout();
                return false;
            }
        } catch (error) {
            console.error('Token validation error:', error);
            this.logout();
            return false;
        }
    },

    /**
     * Log out and clear stored token
     */
    logout() {
        localStorage.removeItem('survivorAuthToken');
    },

    /**
     * Get the stored authentication token
     * @returns {string|null}
     */
    getToken() {
        return localStorage.getItem('survivorAuthToken');
    }
};
