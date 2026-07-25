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

const documentView = document.querySelector('[data-document-id][data-document-revision]');
if (documentView) {
  const documentId = documentView.dataset.documentId;
  let revision = documentView.dataset.documentRevision;
  let reloading = false;

  window.setInterval(async () => {
    try {
      const response = await fetch(`/revisions/${encodeURIComponent(documentId)}`, { cache: 'no-store' });
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
