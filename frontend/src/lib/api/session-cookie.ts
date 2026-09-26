/**
 * Name of the httpOnly cookie holding the JWT issued by the Rust API.
 * Shared by the API client (forwards it), the proxy and the Identity session.
 */
export const SESSION_COOKIE = "xetaravel_token";
