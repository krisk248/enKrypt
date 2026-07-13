// Apply the saved or system colour theme before first paint (no flash).
(function () {
  try {
    var saved = localStorage.getItem('enkrypt.theme');
    var dark =
      saved === 'dark' ||
      (!saved && window.matchMedia('(prefers-color-scheme: dark)').matches);
    document.documentElement.classList.toggle('dark', dark);
  } catch (e) {}
})();
