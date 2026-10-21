import { GOOGLE_CLIENT_ID } from "../config";

// Google sign-in in the browser (Google Identity Services, token model): no backend, so there are no
// refresh tokens. An access token lasts about an hour; after that the user signs in again with one click.
// The token is kept in localStorage so the TV and presenter windows share it.

// Full Drive access: quiz folders are created and shared by other people, and later the app writes to them.
// It also covers the Sheets API.
const SCOPE = "https://www.googleapis.com/auth/drive";
const STORAGE_KEY = "quiznight.google";

export const googleEnabled = Boolean(GOOGLE_CLIENT_ID);

// The Drive features are a private beta for invited accounts: they stay hidden until this browser has
// opened the invitation link (#/drive) or signed in once, so other visitors only see the normal way
const INVITE_KEY = "quiznight.driveInvite";
export function hasDriveInvite() {
  if (!googleEnabled) return false;
  try { return localStorage.getItem(INVITE_KEY) === "1" || Boolean(readSession()?.user); } catch { return false; }
}
export function acceptDriveInvite() {
  try { localStorage.setItem(INVITE_KEY, "1"); } catch { /* storage blocked: shown for this visit only */ }
}

export class GoogleAuthError extends Error {}
export class GoogleApiError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

// { token, expires, user: { name, email, photo } }
function readSession() {
  try { return JSON.parse(localStorage.getItem(STORAGE_KEY)) || null; } catch { return null; }
}

function writeSession(session) {
  try {
    if (session) localStorage.setItem(STORAGE_KEY, JSON.stringify(session));
    else localStorage.removeItem(STORAGE_KEY);
  } catch { /* storage blocked: the session lasts until reload */ }
  memorySession = session;
  listeners.forEach((fn) => fn());
}

let memorySession = readSession();
const listeners = new Set();

// Other windows signing in or out
if (typeof window !== "undefined") {
  window.addEventListener("storage", (e) => {
    if (e.key !== STORAGE_KEY) return;
    memorySession = readSession();
    listeners.forEach((fn) => fn());
  });
}

export function subscribeGoogle(fn) {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

export const getSession = () => memorySession;

// A token with at least a minute left, or null
export function currentToken() {
  const s = memorySession;
  return s?.token && s.expires - Date.now() > 60_000 ? s.token : null;
}

let gisPromise = null;
let tokenClient = null;
let pending = null; // { resolve, reject } of the sign-in in progress

// Load the Google script ahead of time: the sign-in popup must open straight from the click
export function loadGoogle() {
  if (!googleEnabled) return Promise.resolve(false);
  if (!gisPromise) {
    gisPromise = new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = "https://accounts.google.com/gsi/client";
      script.async = true;
      script.onload = () => {
        tokenClient = window.google.accounts.oauth2.initTokenClient({
          client_id: GOOGLE_CLIENT_ID,
          scope: SCOPE,
          callback: onToken,
          error_callback: (err) => finish(null, new GoogleAuthError(err?.type || "popup_closed")),
        });
        resolve(true);
      };
      script.onerror = () => { gisPromise = null; reject(new Error("Google sign-in could not be loaded")); };
      document.head.appendChild(script);
    });
  }
  return gisPromise;
}

function finish(value, error) {
  const p = pending;
  pending = null;
  if (!p) return;
  if (error) p.reject(error);
  else p.resolve(value);
}

async function onToken(response) {
  if (response.error) return finish(null, new GoogleAuthError(response.error));
  // The consent screen lets people untick Drive access
  if (!window.google.accounts.oauth2.hasGrantedAllScopes(response, SCOPE)) return finish(null, new GoogleAuthError("scope"));
  const token = response.access_token;
  const expires = Date.now() + Number(response.expires_in || 3600) * 1000;
  try {
    const res = await fetch("https://www.googleapis.com/drive/v3/about?fields=user(displayName,emailAddress,photoLink)", {
      headers: { Authorization: `Bearer ${token}` },
    });
    const { user } = await res.json();
    const session = { token, expires, user: { name: user.displayName, email: user.emailAddress, photo: user.photoLink || null } };
    writeSession(session);
    finish(session);
  } catch (err) {
    finish(null, err);
  }
}

// Call from a click handler; resolves with the session. Signing in again after the token expired skips
// the account chooser when Google still knows the account; chooseAccount always shows it.
export function signIn({ chooseAccount = false } = {}) {
  if (!tokenClient) return loadGoogle().then(() => signIn({ chooseAccount }));
  if (pending) pending.reject(new GoogleAuthError("superseded"));
  return new Promise((resolve, reject) => {
    pending = { resolve, reject };
    const hint = chooseAccount ? null : memorySession?.user?.email;
    tokenClient.requestAccessToken({ prompt: hint ? "" : "select_account", ...(hint && { login_hint: hint }) });
  });
}

export function signOut() {
  const token = memorySession?.token;
  if (token && window.google?.accounts?.oauth2) window.google.accounts.oauth2.revoke(token, () => {});
  writeSession(null);
}

// fetch with the signed-in user's token; GoogleAuthError when there is none or it was rejected
export async function googleFetch(url, options = {}) {
  const token = currentToken();
  if (!token) throw new GoogleAuthError("signed_out");
  let res;
  try {
    res = await fetch(url, { ...options, headers: { ...options.headers, Authorization: `Bearer ${token}` } });
  } catch {
    throw new GoogleApiError(0, "network");
  }
  if (res.status === 401) {
    // Expired or revoked: keep the account so signing in again is one click
    writeSession({ ...memorySession, token: null });
    throw new GoogleAuthError("expired");
  }
  if (!res.ok) {
    let message = `HTTP ${res.status}`;
    try { message = (await res.json()).error?.message || message; } catch { /* not JSON */ }
    throw new GoogleApiError(res.status, message);
  }
  return res;
}

export const googleJson = async (url, options) => (await googleFetch(url, options)).json();
