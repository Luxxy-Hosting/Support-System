import { bytesToString } from '@/lib/size.ts';
import type { TicketAttachment } from '../types/index.ts';

export const SUPPORT_MAX_ATTACHMENTS_PER_MESSAGE = 5;
export const SUPPORT_MAX_IMAGE_BYTES = 12 * 1024 * 1024;
export const SUPPORT_MAX_VIDEO_BYTES = 80 * 1024 * 1024;
export const SUPPORT_MAX_TOTAL_ATTACHMENT_BYTES = 100 * 1024 * 1024;

export const SUPPORT_ALLOWED_IMAGE_TYPES = new Set([
  'image/png',
  'image/jpeg',
  'image/jpg',
  'image/gif',
  'image/webp',
  'image/avif',
]);

export const SUPPORT_ALLOWED_VIDEO_TYPES = new Set([
  'video/mp4',
  'video/webm',
  'video/ogg',
  'video/quicktime',
]);

export const SUPPORT_ATTACHMENT_ACCEPT = [
  ...SUPPORT_ALLOWED_IMAGE_TYPES,
  ...SUPPORT_ALLOWED_VIDEO_TYPES,
].join(',');

export const classifySupportAttachmentType = (contentType: string): 'image' | 'video' | null => {
  if (SUPPORT_ALLOWED_IMAGE_TYPES.has(contentType)) {
    return 'image';
  }

  if (SUPPORT_ALLOWED_VIDEO_TYPES.has(contentType)) {
    return 'video';
  }

  return null;
};

export const formatSupportAttachmentLimitHint = (): string =>
  `Up to ${SUPPORT_MAX_ATTACHMENTS_PER_MESSAGE} files. Images max ${bytesToString(SUPPORT_MAX_IMAGE_BYTES)}, videos max ${bytesToString(SUPPORT_MAX_VIDEO_BYTES)}.`;

export const validateSupportAttachmentSelection = (
  existingFiles: File[],
  incomingFiles: File[],
): { files: File[]; error: string | null } => {
  const nextFiles = [...existingFiles];

  for (const file of incomingFiles) {
    if (nextFiles.length >= SUPPORT_MAX_ATTACHMENTS_PER_MESSAGE) {
      return {
        files: existingFiles,
        error: `You can upload up to ${SUPPORT_MAX_ATTACHMENTS_PER_MESSAGE} attachments per message.`,
      };
    }

    const type = classifySupportAttachmentType(file.type);
    if (!type) {
      return {
        files: existingFiles,
        error: 'Attachments must be PNG, JPEG, GIF, WebP, AVIF, MP4, WebM, OGG, or MOV.',
      };
    }

    const sizeLimit = type === 'image' ? SUPPORT_MAX_IMAGE_BYTES : SUPPORT_MAX_VIDEO_BYTES;
    if (file.size > sizeLimit) {
      return {
        files: existingFiles,
        error: `${type === 'image' ? 'Images' : 'Videos'} must be smaller than ${bytesToString(sizeLimit)}.`,
      };
    }

    const duplicate = nextFiles.some(
      (existing) =>
        existing.name === file.name &&
        existing.size === file.size &&
        existing.lastModified === file.lastModified,
    );

    if (!duplicate) {
      nextFiles.push(file);
    }
  }

  const totalSize = nextFiles.reduce((total, file) => total + file.size, 0);
  if (totalSize > SUPPORT_MAX_TOTAL_ATTACHMENT_BYTES) {
    return {
      files: existingFiles,
      error: `Total attachment size must be smaller than ${bytesToString(SUPPORT_MAX_TOTAL_ATTACHMENT_BYTES)}.`,
    };
  }

  return { files: nextFiles, error: null };
};

export const isImageAttachment = (attachment: TicketAttachment): boolean => attachment.mediaType === 'image';

export const isVideoAttachment = (attachment: TicketAttachment): boolean => attachment.mediaType === 'video';
