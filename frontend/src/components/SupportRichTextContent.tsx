import DOMPurify from 'dompurify';
import { useMemo } from 'react';
import { normalizeStoredMessageHtml } from '../helpers/richText.ts';

interface Props {
  value: string;
}

export default function SupportRichTextContent({ value }: Props) {
  const sanitizedHtml = useMemo(
    () =>
      DOMPurify.sanitize(normalizeStoredMessageHtml(value), {
        USE_PROFILES: { html: true },
      }),
    [value],
  );

  return <div className='support-ticket-richtext-content' dangerouslySetInnerHTML={{ __html: sanitizedHtml }} />;
}
