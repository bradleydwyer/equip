// equip website

// Nav scroll
const nav = document.getElementById('nav');
window.addEventListener('scroll', () => {
  nav.classList.toggle('scrolled', window.scrollY > 20);
}, { passive: true });

// Mobile nav
const toggle = document.querySelector('.nav-toggle');
const links = document.querySelector('.nav-links');
if (toggle) {
  toggle.addEventListener('click', () => links.classList.toggle('open'));
  links.querySelectorAll('a').forEach(a =>
    a.addEventListener('click', () => links.classList.remove('open'))
  );
}

// Terminal animation
function runHeroTerm() {
  const term = document.getElementById('hero-term');
  if (!term) return;
  const lines = term.querySelectorAll('.tl');
  lines.forEach(l => l.classList.remove('vis'));
  lines.forEach(l => {
    setTimeout(() => l.classList.add('vis'), parseInt(l.dataset.d) || 0);
  });
  let max = 0;
  lines.forEach(l => { const d = parseInt(l.dataset.d) || 0; if (d > max) max = d; });
  setTimeout(runHeroTerm, max + 4000);
}
setTimeout(runHeroTerm, 500);

// Reveals
const obs = new IntersectionObserver(entries => {
  entries.forEach(e => {
    if (e.isIntersecting) {
      const siblings = Array.from(e.target.parentElement.querySelectorAll('.reveal'));
      const idx = siblings.indexOf(e.target);
      setTimeout(() => e.target.classList.add('visible'), idx * 70);
      obs.unobserve(e.target);
    }
  });
}, { threshold: 0.1 });
document.querySelectorAll('.reveal').forEach(el => obs.observe(el));

// Copy buttons
document.querySelectorAll('.copy-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    const t = document.getElementById(btn.dataset.target);
    if (!t) return;
    navigator.clipboard.writeText(t.textContent.trim()).then(() => {
      const o = btn.textContent;
      btn.textContent = 'Copied!';
      btn.style.color = 'var(--accent)';
      setTimeout(() => { btn.textContent = o; btn.style.color = ''; }, 2000);
    });
  });
});

// Smooth scroll
document.querySelectorAll('a[href^="#"]').forEach(a => {
  a.addEventListener('click', e => {
    const t = document.querySelector(a.getAttribute('href'));
    if (t) { e.preventDefault(); t.scrollIntoView({ behavior: 'smooth' }); }
  });
});
