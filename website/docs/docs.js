// equip docs

// Active sidebar link
const currentPage = location.pathname.split('/').pop() || 'index.html';
document.querySelectorAll('.docs-nav-link').forEach(a => {
  const href = a.getAttribute('href').replace('./', 'index.html');
  if (href === currentPage) a.classList.add('active');
});

// Build table of contents
function buildTOC() {
  const toc = document.getElementById('toc');
  if (!toc) return;
  const headings = document.querySelectorAll('.docs-content h2, .docs-content h3');
  if (headings.length === 0) return;

  const title = document.createElement('div');
  title.className = 'docs-toc-title';
  title.textContent = 'On this page';
  toc.appendChild(title);

  headings.forEach(h => {
    if (!h.id) h.id = h.textContent.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
    const a = document.createElement('a');
    a.href = '#' + h.id;
    a.textContent = h.textContent;
    if (h.tagName === 'H3') a.classList.add('toc-h3');
    a.addEventListener('click', e => {
      e.preventDefault();
      h.scrollIntoView({ behavior: 'smooth' });
      history.replaceState(null, '', '#' + h.id);
    });
    toc.appendChild(a);
  });
}
buildTOC();

// Scroll spy for TOC
function setupScrollSpy() {
  const toc = document.getElementById('toc');
  if (!toc) return;
  const links = toc.querySelectorAll('a');
  if (links.length === 0) return;

  const headings = [];
  links.forEach(a => {
    const id = a.getAttribute('href').slice(1);
    const el = document.getElementById(id);
    if (el) headings.push({ el, link: a });
  });

  function update() {
    let current = headings[0];
    for (const h of headings) {
      if (h.el.getBoundingClientRect().top <= 100) current = h;
    }
    links.forEach(a => a.classList.remove('active'));
    if (current) current.link.classList.add('active');
  }

  window.addEventListener('scroll', update, { passive: true });
  update();
}
setupScrollSpy();

// Mobile sidebar toggle
const sidebar = document.getElementById('sidebar');
const overlay = document.querySelector('.docs-sidebar-overlay');
const navToggle = document.querySelector('.nav-toggle');

if (navToggle && sidebar) {
  navToggle.addEventListener('click', () => {
    sidebar.classList.toggle('open');
    if (overlay) overlay.classList.toggle('open');
  });
}
if (overlay) {
  overlay.addEventListener('click', () => {
    sidebar.classList.remove('open');
    overlay.classList.remove('open');
  });
}
document.querySelectorAll('.docs-nav-link').forEach(a => {
  a.addEventListener('click', () => {
    if (sidebar) sidebar.classList.remove('open');
    if (overlay) overlay.classList.remove('open');
  });
});
