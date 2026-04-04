const htmlTagPattern = /<\/?[a-z][\s\S]*>/i;

const escapeHtml = (value: string): string =>
  value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');

export const isRichTextEmpty = (value: string | null | undefined): boolean => {
  if (!value) {
    return true;
  }

  if (typeof window === 'undefined') {
    return value.replace(/<[^>]+>/g, '').replace(/&nbsp;/g, ' ').trim().length === 0;
  }

  const container = window.document.createElement('div');
  container.innerHTML = value;
  return (container.textContent ?? '').replace(/\u00a0/g, ' ').trim().length === 0;
};

export const normalizeStoredMessageHtml = (value: string): string => {
  if (!value) {
    return '<p></p>';
  }

  if (htmlTagPattern.test(value)) {
    return value;
  }

  const paragraphs = value
    .split(/\n{2,}/)
    .map((paragraph) => paragraph.trim())
    .filter(Boolean)
    .map((paragraph) => `<p>${escapeHtml(paragraph).replace(/\n/g, '<br />')}</p>`);

  return paragraphs.length ? paragraphs.join('') : `<p>${escapeHtml(value).replace(/\n/g, '<br />')}</p>`;
};
