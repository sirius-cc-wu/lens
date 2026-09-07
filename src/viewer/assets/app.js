for (const image of document.querySelectorAll('[data-diagram]')) {
  const revealFailure = () => {
    const figure = image.closest('.diagram');
    figure.querySelector('.diagram-error').hidden = false;
    figure.querySelector('[data-diagram-retry]').hidden = false;
    figure.querySelector('.diagram-source').open = true;
  };
  const retry = image.closest('.diagram').querySelector('[data-diagram-retry]');
  retry.addEventListener('click', () => {
    image.closest('.diagram').querySelector('.diagram-error').hidden = true;
    retry.hidden = true;
    const retryUrl = new URL(image.src, window.location.origin);
    retryUrl.searchParams.set('retry', Date.now().toString());
    image.src = retryUrl.toString();
  });
  image.addEventListener('error', revealFailure);
  if (image.complete && image.naturalWidth === 0) {
    revealFailure();
  }
}

if (typeof mermaid !== 'undefined') {
  mermaid.initialize({
    startOnLoad: false,
    securityLevel: 'strict',
    theme: 'neutral',
    suppressErrorRendering: true,
    secure: [
      'secure',
      'securityLevel',
      'startOnLoad',
      'maxTextSize',
      'suppressErrorRendering',
      'maxEdges',
      'fontFamily',
      'themeCSS',
      'altFontFamily',
      'themeVariables',
    ],
  });

  let mermaidCounter = 0;
  for (const container of document.querySelectorAll('[data-mermaid-container]')) {
    const target = container.querySelector('.mermaid-target');
    const errorMsg = container.querySelector('.diagram-error');
    const details = container.querySelector('.diagram-source');
    const source = details ? details.querySelector('code').textContent : '';
    const id = `mermaid-svg-${++mermaidCounter}`;

    try {
      const renderPromise = mermaid.render(id, source);
      if (renderPromise && typeof renderPromise.then === 'function') {
        renderPromise
          .then(({ svg }) => {
            target.innerHTML = svg;
          })
          .catch((_error) => {
            target.hidden = true;
            errorMsg.hidden = false;
            details.open = true;
          });
      }
    } catch (_error) {
      target.hidden = true;
      errorMsg.hidden = false;
      details.open = true;
    }
  }
}

const documentView = document.querySelector('[data-document-id][data-document-revision]');
if (documentView) {
  const documentId = documentView.dataset.documentId;
  const sessionToken = documentView.dataset.sessionToken;
  let revision = documentView.dataset.documentRevision;
  let reloading = false;

  window.setInterval(async () => {
    try {
      const tokenParam = sessionToken ? `?token=${encodeURIComponent(sessionToken)}` : '';
      const response = await fetch(`/revisions/${encodeURIComponent(documentId)}${tokenParam}`, { cache: 'no-store' });
      if (!response.ok) return;
      const currentRevision = await response.text();
      if (currentRevision !== revision && !reloading) {
        reloading = true;
        window.location.reload();
      }
    } catch {
      // Retain the readable document and try again on the next interval.
    }
  }, 500);
}

document.addEventListener('click', (event) => {
  const link = event.target.closest('a[href]');
  if (!link) return;
  const href = link.getAttribute('href');
  if (!href || href.startsWith('#') || href.startsWith('javascript:') || href.startsWith('vscode:') || href.startsWith('mailto:')) {
    return;
  }
  const documentView = document.querySelector('[data-session-token]');
  const token = documentView ? documentView.dataset.sessionToken : null;
  if (!token) return;

  try {
    const url = new URL(link.href, window.location.href);
    if (url.origin === window.location.origin && !url.searchParams.has('token')) {
      url.searchParams.set('token', token);
      link.href = url.toString();
    }
  } catch {
    // Retain authored destination on URL parse failure.
  }
});
