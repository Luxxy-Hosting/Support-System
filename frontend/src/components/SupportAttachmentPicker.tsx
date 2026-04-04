import { Group, Stack, Text } from '@mantine/core';
import { useRef } from 'react';
import Button from '@/elements/Button.tsx';
import { bytesToString } from '@/lib/size.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import {
  classifySupportAttachmentType,
  formatSupportAttachmentLimitHint,
  SUPPORT_ATTACHMENT_ACCEPT,
  validateSupportAttachmentSelection,
} from '../helpers/attachments.ts';

interface Props {
  files: File[];
  disabled?: boolean;
  onChange: (files: File[]) => void;
}

export default function SupportAttachmentPicker({ files, disabled = false, onChange }: Props) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  const { addToast } = useToast();

  const handleSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const incomingFiles = Array.from(event.currentTarget.files ?? []);
    if (!incomingFiles.length) {
      return;
    }

    const nextState = validateSupportAttachmentSelection(files, incomingFiles);
    if (nextState.error) {
      addToast(nextState.error, 'error');
    } else {
      onChange(nextState.files);
    }

    event.currentTarget.value = '';
  };

  const removeFile = (targetIndex: number) => {
    onChange(files.filter((_, index) => index !== targetIndex));
  };

  return (
    <Stack gap='xs'>
      <input
        ref={inputRef}
        type='file'
        hidden
        multiple
        accept={SUPPORT_ATTACHMENT_ACCEPT}
        disabled={disabled}
        onChange={handleSelect}
      />

      <Group justify='space-between' align='center'>
        <Group gap='sm'>
          <Button size='xs' variant='default' disabled={disabled} onClick={() => inputRef.current?.click()}>
            Add Media
          </Button>
          {files.length > 0 && (
            <Button size='xs' variant='subtle' color='gray' disabled={disabled} onClick={() => onChange([])}>
              Clear All
            </Button>
          )}
        </Group>

        <Text size='xs' c='dimmed'>
          {formatSupportAttachmentLimitHint()}
        </Text>
      </Group>

      {files.length > 0 && (
        <div className='support-ticket-pending-attachments'>
          {files.map((file, index) => {
            const mediaType = classifySupportAttachmentType(file.type) ?? 'file';

            return (
              <div key={`${file.name}-${file.size}-${file.lastModified}`} className='support-ticket-pending-attachment'>
                <div className='support-ticket-pending-attachment-body'>
                  <Text size='sm' fw={500} truncate>
                    {file.name}
                  </Text>
                  <Text size='xs' c='dimmed'>
                    {mediaType === 'video' ? 'Video' : 'Image'} • {bytesToString(file.size)}
                  </Text>
                </div>

                <Button size='xs' variant='subtle' color='gray' disabled={disabled} onClick={() => removeFile(index)}>
                  Remove
                </Button>
              </div>
            );
          })}
        </div>
      )}
    </Stack>
  );
}
