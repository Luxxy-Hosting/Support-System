import { Text } from '@mantine/core';
import { useMemo, useState } from 'react';
import Button from '@/elements/Button.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import { bytesToString } from '@/lib/size.ts';
import { isImageAttachment, isVideoAttachment } from '../helpers/attachments.ts';
import type { TicketAttachment } from '../types/index.ts';

interface Props {
  attachments: TicketAttachment[];
}

export default function SupportMessageAttachments({ attachments }: Props) {
  const [activeAttachment, setActiveAttachment] = useState<TicketAttachment | null>(null);
  const [zoomed, setZoomed] = useState(false);

  const hasAttachments = attachments.length > 0;
  const activeIsImage = useMemo(
    () => (activeAttachment ? isImageAttachment(activeAttachment) : false),
    [activeAttachment],
  );

  if (!hasAttachments) {
    return null;
  }

  const closeModal = () => {
    setActiveAttachment(null);
    setZoomed(false);
  };

  return (
    <>
      <div className='support-ticket-attachments-grid'>
        {attachments.map((attachment) => (
          <button
            key={attachment.uuid}
            type='button'
            className='support-ticket-attachment-tile'
            onClick={() => {
              setActiveAttachment(attachment);
              setZoomed(false);
            }}
          >
            <div className='support-ticket-attachment-preview'>
              {isImageAttachment(attachment) ? (
                <img src={attachment.url} alt={attachment.originalName} className='support-ticket-attachment-preview-media' />
              ) : isVideoAttachment(attachment) ? (
                <video
                  src={attachment.url}
                  className='support-ticket-attachment-preview-media'
                  muted
                  preload='metadata'
                />
              ) : null}
            </div>

            <div className='support-ticket-attachment-meta'>
              <Text size='sm' fw={500} truncate>
                {attachment.originalName}
              </Text>
              <Text size='xs' c='dimmed'>
                {attachment.mediaType === 'video' ? 'Video' : 'Image'} • {bytesToString(attachment.size)}
              </Text>
            </div>
          </button>
        ))}
      </div>

      <Modal
        opened={activeAttachment !== null}
        onClose={closeModal}
        title={activeAttachment?.originalName ?? 'Attachment Preview'}
        size='xl'
      >
        {activeAttachment && (
          <>
            <div className={`support-ticket-lightbox ${zoomed ? 'support-ticket-lightbox--zoomed' : ''}`.trim()}>
              {activeIsImage ? (
                <button
                  type='button'
                  className='support-ticket-lightbox-image-button'
                  onClick={() => setZoomed((current) => !current)}
                >
                  <img
                    src={activeAttachment.url}
                    alt={activeAttachment.originalName}
                    className='support-ticket-lightbox-media support-ticket-lightbox-media-image'
                  />
                </button>
              ) : (
                <video
                  src={activeAttachment.url}
                  controls
                  autoPlay
                  className='support-ticket-lightbox-media support-ticket-lightbox-media-video'
                />
              )}
            </div>

            <Text size='xs' c='dimmed' mt='sm'>
              {activeIsImage
                ? 'Click the image to toggle zoom, or open the original in a new tab for browser zoom controls.'
                : 'Use the built-in video controls or open the original in a new tab.'}
            </Text>

            <ModalFooter>
              <Button component='a' href={activeAttachment.url} target='_blank' rel='noreferrer' variant='light'>
                Open Original
              </Button>
              <Button variant='default' onClick={closeModal}>
                Close
              </Button>
            </ModalFooter>
          </>
        )}
      </Modal>
    </>
  );
}
