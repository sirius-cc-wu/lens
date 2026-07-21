const markDiagramDisabled = (figure) => {
  const image = figure.querySelector('[data-diagram]');
  if (image) image.removeAttribute('src');
  figure.querySelector('.diagram-error').hidden = true;
  figure.querySelector('[data-diagram-retry]').hidden = true;
  figure.querySelector('[data-diagram-disabled]').hidden = false;
  figure.querySelector('.diagram-source').open = true;
};

const navigationControl = document.querySelector('[data-document-navigation-control]');
const navigationToggle = document.querySelector('[data-document-navigation-toggle]');
const navigationPane = document.querySelector('#document-navigation');
const documentLayout = document.querySelector('main');
if (navigationControl && navigationToggle && navigationPane && documentLayout) {
  const navigationPaneStateKey = 'lens.documentNavigationCollapsed';
  const setNavigationPaneCollapsed = (collapsed) => {
    navigationPane.hidden = collapsed;
    documentLayout.dataset.documentNavigationCollapsed = String(collapsed);
    navigationToggle.setAttribute('aria-expanded', String(!collapsed));
    navigationToggle.textContent = collapsed ? 'Show documents' : 'Hide documents';
  };
  let collapsed = false;
  try {
    collapsed = sessionStorage.getItem(navigationPaneStateKey) === 'true';
  } catch {
    // Keep the navigation pane visible when browser session storage is unavailable.
  }
  setNavigationPaneCollapsed(collapsed);
  navigationControl.hidden = false;
  navigationToggle.addEventListener('click', () => {
    collapsed = !navigationPane.hidden;
    setNavigationPaneCollapsed(collapsed);
    try {
      sessionStorage.setItem(navigationPaneStateKey, String(collapsed));
    } catch {
      // Retain the current page's visibility when browser session storage is unavailable.
    }
  });
}

for (const image of document.querySelectorAll('[data-diagram]')) {
  const revealFailure = () => {
    const figure = image.closest('.diagram');
    if (document.documentElement.dataset.diagramRenderingDisabled === 'true') {
      markDiagramDisabled(figure);
      return;
    }
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

const disableRenderer = document.querySelector('[data-disable-renderer]');
if (disableRenderer) {
  disableRenderer.addEventListener('click', async () => {
    disableRenderer.disabled = true;
    try {
      const response = await fetch('/renderer/disable', { method: 'POST' });
      if (!response.ok) throw new Error('disable failed');
      document.documentElement.dataset.diagramRenderingDisabled = 'true';
      document.querySelector('[data-renderer-status]').textContent =
        'Diagram rendering is disabled for this viewing session.';
      for (const figure of document.querySelectorAll('[data-diagram-container]')) {
        markDiagramDisabled(figure);
      }
      disableRenderer.remove();
    } catch {
      disableRenderer.disabled = false;
    }
  });
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
