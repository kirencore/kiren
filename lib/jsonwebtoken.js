// Kiren JWT Compatibility Layer
// Basic JWT decode support (no verification)

// Base64 URL decode
function base64UrlDecode(str) {
  // Add padding if needed
  str = str.replace(/-/g, '+').replace(/_/g, '/');
  while (str.length % 4) str += '=';

  // Decode base64
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
  let output = '';

  for (let i = 0; i < str.length; i += 4) {
    const enc1 = chars.indexOf(str.charAt(i));
    const enc2 = chars.indexOf(str.charAt(i + 1));
    const enc3 = chars.indexOf(str.charAt(i + 2));
    const enc4 = chars.indexOf(str.charAt(i + 3));

    const chr1 = (enc1 << 2) | (enc2 >> 4);
    const chr2 = ((enc2 & 15) << 4) | (enc3 >> 2);
    const chr3 = ((enc3 & 3) << 6) | enc4;

    output += String.fromCharCode(chr1);
    if (enc3 !== 64) output += String.fromCharCode(chr2);
    if (enc4 !== 64) output += String.fromCharCode(chr3);
  }

  return output;
}

function decode(token, options = {}) {
  if (!token) return null;

  try {
    const parts = token.split('.');
    if (parts.length !== 3) return null;

    const payload = base64UrlDecode(parts[1]);
    const decoded = JSON.parse(payload);

    if (options.complete) {
      const header = JSON.parse(base64UrlDecode(parts[0]));
      return {
        header,
        payload: decoded,
        signature: parts[2]
      };
    }

    return decoded;
  } catch (e) {
    return null;
  }
}

function verify(token, secret, options = {}) {
  // Basic decode without actual verification
  // For production, implement proper HMAC verification
  const decoded = decode(token);
  if (!decoded) {
    throw new Error('Invalid token');
  }

  // Check expiration
  if (decoded.exp && Date.now() >= decoded.exp * 1000) {
    throw new Error('Token expired');
  }

  return decoded;
}

function sign(payload, secret, options = {}) {
  // Basic JWT creation (HS256)
  const header = { alg: 'HS256', typ: 'JWT' };

  const now = Math.floor(Date.now() / 1000);
  const tokenPayload = {
    ...payload,
    iat: now
  };

  if (options.expiresIn) {
    let exp = now;
    if (typeof options.expiresIn === 'number') {
      exp += options.expiresIn;
    } else if (typeof options.expiresIn === 'string') {
      const match = options.expiresIn.match(/^(\d+)([smhd])$/);
      if (match) {
        const value = parseInt(match[1]);
        const unit = match[2];
        switch (unit) {
          case 's': exp += value; break;
          case 'm': exp += value * 60; break;
          case 'h': exp += value * 3600; break;
          case 'd': exp += value * 86400; break;
        }
      }
    }
    tokenPayload.exp = exp;
  }

  // Encode header and payload
  const headerB64 = btoa(JSON.stringify(header));
  const payloadB64 = btoa(JSON.stringify(tokenPayload));

  // For now, create a simple signature (not cryptographically secure)
  // In production, use proper HMAC-SHA256
  const signature = btoa(secret + headerB64 + payloadB64).slice(0, 43);

  return `${headerB64}.${payloadB64}.${signature}`;
}

// Base64 encode
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

  // Make URL-safe
  return output.replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

module.exports = {
  decode,
  verify,
  sign
};
