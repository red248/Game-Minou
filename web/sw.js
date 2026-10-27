// Offline support: serve from cache, refresh the cache in the background.
const CACHE = 'minou-__VERSION__';
const FILES = ['./', './index.html', './style.css?v=__VERSION__', './minou.js?v=__VERSION__',
  './minou_bg.wasm?v=__VERSION__', './manifest.webmanifest', './icon-180.png', './icon-192.png', './icon-512.png'];

self.addEventListener('install', e => {
  e.waitUntil(caches.open(CACHE).then(c => c.addAll(FILES)).then(() => self.skipWaiting()));
});

self.addEventListener('activate', e => {
  e.waitUntil(caches.keys()
    .then(keys => Promise.all(keys.filter(k => k !== CACHE).map(k => caches.delete(k))))
    .then(() => self.clients.claim()));
});

self.addEventListener('fetch', e => {
  if (e.request.method !== 'GET') return;
  const isPage = e.request.mode === 'navigate';
  if (isPage) {
    // Network first for the page so updates arrive quickly.
    e.respondWith(fetch(e.request).then(r => {
      caches.open(CACHE).then(c => c.put('./', r.clone()));
      return r;
    }).catch(() => caches.match('./')));
    return;
  }
  e.respondWith(caches.match(e.request).then(hit => hit || fetch(e.request)));
});
