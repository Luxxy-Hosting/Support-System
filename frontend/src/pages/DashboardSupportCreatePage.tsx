import { Group, Select as MantineSelect, Stack, Text, Title } from '@mantine/core';
import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import AccountContentContainer from '@/elements/containers/AccountContentContainer.tsx';
import ScreenBlock from '@/elements/ScreenBlock.tsx';
import Spinner from '@/elements/Spinner.tsx';
import TextArea from '@/elements/input/TextArea.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import { useToast } from '@/providers/ToastProvider.tsx';
import { createClientTicket, createClientTicketUpload, getClientBootstrap } from '../api/client.ts';
import SupportAttachmentPicker from '../components/SupportAttachmentPicker.tsx';
import SupportRichTextEditor from '../components/SupportRichTextEditor.tsx';
import { buildServerOptionLabel } from '../helpers/tickets.ts';
import { isRichTextEmpty } from '../helpers/richText.ts';
import type { ClientTicketBootstrap } from '../types/index.ts';

export default function DashboardSupportCreatePage() {
  const navigate = useNavigate();
  const { addToast } = useToast();

  const [bootstrap, setBootstrap] = useState<ClientTicketBootstrap | null>(null);
  const [fatalError, setFatalError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  const [serverUuid, setServerUuid] = useState<string | null>(null);
  const [categoryUuid, setCategoryUuid] = useState<string | null>(null);
  const [subject, setSubject] = useState('');
  const [message, setMessage] = useState('');
  const [additionalContext, setAdditionalContext] = useState('');
  const [attachments, setAttachments] = useState<File[]>([]);

  useEffect(() => {
    let mounted = true;

    getClientBootstrap()
      .then((response) => {
        if (!mounted) return;
        setBootstrap(response);
      })
      .catch((error) => {
        if (!mounted) return;
        setFatalError(httpErrorToHuman(error));
      })
      .finally(() => {
        if (mounted) {
          setLoading(false);
        }
      });

    return () => {
      mounted = false;
    };
  }, []);

  const serverOptions = useMemo(
    () =>
      (bootstrap?.servers ?? []).map((server) => ({
        value: server.uuid,
        label: buildServerOptionLabel(server),
      })),
    [bootstrap?.servers],
  );

  const categoryOptions = useMemo(
    () =>
      (bootstrap?.categories ?? [])
        .filter((category) => category.enabled)
        .map((category) => ({ value: category.uuid, label: category.name })),
    [bootstrap?.categories],
  );

  const handleSubmit = async () => {
    try {
      setSubmitting(true);
      const normalizedMessage = isRichTextEmpty(message) ? '' : message;
      const payload = {
        serverUuid: serverUuid || undefined,
        categoryUuid: categoryUuid || undefined,
        subject,
        message: normalizedMessage,
        metadata: additionalContext.trim().length
          ? {
              additionalContext: additionalContext.trim(),
            }
          : undefined,
      };

      const ticket = attachments.length > 0
        ? await createClientTicketUpload({
            ...payload,
            files: attachments,
          })
        : await createClientTicket(payload);

      addToast('Ticket created successfully.', 'success');
      navigate(`/account/support/${ticket.ticket.uuid}`);
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setSubmitting(false);
    }
  };

  if (loading) {
    return (
      <AccountContentContainer title='Create Ticket'>
        <Spinner.Centered />
      </AccountContentContainer>
    );
  }

  if (fatalError || !bootstrap) {
    return (
      <AccountContentContainer title='Create Ticket'>
        <ScreenBlock title='Support Unavailable' content={fatalError ?? 'Unable to load support settings.'} />
      </AccountContentContainer>
    );
  }

  return (
    <AccountContentContainer title='Create Ticket'>
      <Group justify='space-between' align='end' mb='md'>
        <div>
          <Title order={1} c='white'>
            Create Ticket
          </Title>
          <Text c='dimmed'>
            Create a general support request or attach it to one of your servers.
          </Text>
        </div>
        <Button variant='light' color='gray' onClick={() => navigate('/account/support')}>
          Back to Tickets
        </Button>
      </Group>

      <Stack gap='md'>
        <MantineSelect
          label='Linked Server'
          description='Choose a server if this issue is tied to a specific service, or leave it empty for a general ticket.'
          data={serverOptions}
          value={serverUuid}
          allowDeselect
          clearable
          searchable
          onChange={setServerUuid}
        />

        {bootstrap.settings.categoriesEnabled && categoryOptions.length > 0 && (
          <MantineSelect
            label='Category'
            data={categoryOptions}
            value={categoryUuid}
            allowDeselect
            clearable
            searchable
            onChange={setCategoryUuid}
          />
        )}

        <TextInput label='Subject' value={subject} onChange={(event) => setSubject(event.currentTarget.value)} />

        <div>
          <Text size='sm' fw={500} mb={6}>
            Message
          </Text>
          <SupportRichTextEditor
            value={message}
            placeholder='Describe the issue, what you expected, and what happened instead.'
            disabled={submitting}
            onChange={setMessage}
          />
        </div>

        <SupportAttachmentPicker files={attachments} disabled={submitting} onChange={setAttachments} />

        <TextArea
          label={serverUuid ? 'Additional Server Context' : 'Additional Context'}
          description='Optional structured context that staff should keep with this ticket.'
          autosize
          minRows={4}
          value={additionalContext}
          onChange={(event) => setAdditionalContext(event.currentTarget.value)}
        />

        <Group justify='flex-end'>
          <Button variant='default' onClick={() => navigate('/account/support')}>
            Cancel
          </Button>
          <Button
            color='blue'
            loading={submitting}
            disabled={!subject.trim() || (isRichTextEmpty(message) && attachments.length === 0)}
            onClick={handleSubmit}
          >
            Submit Ticket
          </Button>
        </Group>
      </Stack>
    </AccountContentContainer>
  );
}
