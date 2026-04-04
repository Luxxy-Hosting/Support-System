import { Divider, Group, Stack, Text, Title } from '@mantine/core';
import { type ReactNode, useEffect, useMemo, useState } from 'react';
import { NavLink, useNavigate, useParams } from 'react-router';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import AccountContentContainer from '@/elements/containers/AccountContentContainer.tsx';
import ScreenBlock from '@/elements/ScreenBlock.tsx';
import Spinner from '@/elements/Spinner.tsx';
import { useToast } from '@/providers/ToastProvider.tsx';
import { addClientReply, addClientReplyUpload, getClientBootstrap, getClientTicket, updateClientTicketStatus } from '../api/client.ts';
import SupportAttachmentPicker from '../components/SupportAttachmentPicker.tsx';
import SupportRichTextEditor from '../components/SupportRichTextEditor.tsx';
import TicketConversation from '../components/TicketConversation.tsx';
import TicketPriorityBadge from '../components/TicketPriorityBadge.tsx';
import TicketStatusBadge from '../components/TicketStatusBadge.tsx';
import {
  describeLinkedServer,
  extractClientMetadata,
  formatTicketDateTime,
  humanizeTicketActor,
  humanizeTicketStatus,
} from '../helpers/tickets.ts';
import { isRichTextEmpty } from '../helpers/richText.ts';
import type { ClientTicketBootstrap, TicketDetail } from '../types/index.ts';

function SidebarDetailRow({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className='support-ticket-sidebar-row'>
      <Text size='xs' c='dimmed' className='support-ticket-sidebar-row-label'>
        {label}
      </Text>
      <Text size='sm' fw={500} className='support-ticket-sidebar-row-value'>
        {value}
      </Text>
    </div>
  );
}

function SidebarValueLink({ to, children }: { to: string; children: ReactNode }) {
  return (
    <NavLink to={to} className='support-ticket-inline-link'>
      {children}
    </NavLink>
  );
}

const humanizeMetadataKey = (value: string): string =>
  value
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/[_-]+/g, ' ')
    .split(' ')
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(' ');

const hasMetadataValue = (value: unknown): boolean => {
  if (value === null || value === undefined) {
    return false;
  }

  if (typeof value === 'string') {
    return value.trim().length > 0;
  }

  if (Array.isArray(value)) {
    return value.length > 0;
  }

  if (typeof value === 'object') {
    return Object.keys(value as Record<string, unknown>).length > 0;
  }

  return true;
};

const formatMetadataValue = (value: unknown): string => {
  if (typeof value === 'string') {
    return value;
  }

  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value);
  }

  return JSON.stringify(value);
};

export default function DashboardSupportTicketPage() {
  const navigate = useNavigate();
  const { ticket: ticketUuid } = useParams();
  const { addToast } = useToast();

  const [bootstrap, setBootstrap] = useState<ClientTicketBootstrap | null>(null);
  const [detail, setDetail] = useState<TicketDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [fatalError, setFatalError] = useState<string | null>(null);
  const [replyBody, setReplyBody] = useState('');
  const [replyAttachments, setReplyAttachments] = useState<File[]>([]);
  const [replyLoading, setReplyLoading] = useState(false);
  const [actionLoading, setActionLoading] = useState(false);

  useEffect(() => {
    if (!ticketUuid) {
      setFatalError('Ticket not found.');
      setLoading(false);
      return;
    }

    let mounted = true;
    setLoading(true);

    Promise.all([getClientBootstrap(), getClientTicket(ticketUuid)])
      .then(([bootstrapResponse, detailResponse]) => {
        if (!mounted) return;
        setBootstrap(bootstrapResponse);
        setDetail(detailResponse);
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
  }, [ticketUuid]);

  const clientMetadata = useMemo(() => extractClientMetadata(detail?.metadata ?? null), [detail?.metadata]);
  const clientMetadataEntries = useMemo(
    () =>
      Object.entries(clientMetadata)
        .filter(([, value]) => hasMetadataValue(value))
        .map(([key, value]) => ({
          label: humanizeMetadataKey(key),
          value: formatMetadataValue(value),
        })),
    [clientMetadata],
  );
  const hasClientMetadata = clientMetadataEntries.length > 0;
  const linkedServerPath = detail?.ticket.linkedServer.currentUuidShort
    ? `/server/${detail.ticket.linkedServer.currentUuidShort}`
    : null;
  const canReply = detail && bootstrap
    ? detail.ticket.status !== 'closed' || bootstrap.settings.allowReplyOnClosed
    : false;
  const canClose = detail && bootstrap
    ? bootstrap.settings.allowClientClose && detail.ticket.status !== 'closed'
    : false;
  const canReopen = detail?.ticket.status === 'closed';

  const refreshTicket = async (nextTicket: TicketDetail) => {
    setDetail(nextTicket);
    setReplyBody('');
    setReplyAttachments([]);
  };

  const handleReply = async () => {
    if (!ticketUuid || (isRichTextEmpty(replyBody) && replyAttachments.length === 0)) {
      return;
    }

    try {
      setReplyLoading(true);
      const nextTicket = replyAttachments.length > 0
        ? await addClientReplyUpload(ticketUuid, {
            body: isRichTextEmpty(replyBody) ? '' : replyBody,
            files: replyAttachments,
          })
        : await addClientReply(ticketUuid, replyBody);
      await refreshTicket(nextTicket);
      addToast('Reply sent.', 'success');
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setReplyLoading(false);
    }
  };

  const handleStatusChange = async (status: 'closed' | 'waiting_on_staff') => {
    if (!ticketUuid) {
      return;
    }

    try {
      setActionLoading(true);
      const nextTicket = await updateClientTicketStatus(ticketUuid, status);
      await refreshTicket(nextTicket);
      addToast(status === 'closed' ? 'Ticket closed.' : 'Ticket reopened.', 'success');
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setActionLoading(false);
    }
  };

  if (loading) {
    return (
      <AccountContentContainer title='Ticket'>
        <Spinner.Centered />
      </AccountContentContainer>
    );
  }

  if (!detail || fatalError) {
    return (
      <AccountContentContainer title='Ticket'>
        <ScreenBlock title='Ticket Unavailable' content={fatalError ?? 'Unable to load ticket details.'} />
      </AccountContentContainer>
    );
  }

  return (
    <AccountContentContainer title={detail.ticket.subject}>
      <Group justify='space-between' align='start' mb='md'>
        <div>
          <Title order={1} c='white'>
            {detail.ticket.subject}
          </Title>
          <Text c='dimmed'>
            Created {formatTicketDateTime(detail.ticket.created)} by @{detail.ticket.creator.username}
          </Text>
        </div>
        <Group>
          <Button variant='light' color='gray' onClick={() => navigate('/account/support')}>
            Back to Tickets
          </Button>
          {canClose && (
            <Button color='red' loading={actionLoading} onClick={() => handleStatusChange('closed')}>
              Close Ticket
            </Button>
          )}
          {canReopen && (
            <Button color='blue' loading={actionLoading} onClick={() => handleStatusChange('waiting_on_staff')}>
              Reopen Ticket
            </Button>
          )}
        </Group>
      </Group>

      <div className='support-ticket-detail-page'>
        <div className='support-ticket-workspace'>
          <div className='support-ticket-detail-grid'>
            <div className='support-ticket-detail-main'>
              <Card className='support-ticket-panel support-ticket-workspace-card'>
                <Text fw={600} mb='md'>
                  Conversation
                </Text>
                <div className='support-ticket-thread-body'>
                  <TicketConversation messages={detail.messages} scrollable />
                </div>
                <div className='support-ticket-composer-section'>
                  <Divider mb='md' />
                  <SupportRichTextEditor
                    value={replyBody}
                    disabled={!canReply}
                    placeholder={canReply ? 'Write your reply...' : 'Replies are disabled for this ticket.'}
                    onChange={setReplyBody}
                  />
                  <SupportAttachmentPicker
                    files={replyAttachments}
                    disabled={!canReply || replyLoading}
                    onChange={setReplyAttachments}
                  />
                  <Group justify='flex-end' mt='md'>
                    <Button
                      color='blue'
                      loading={replyLoading}
                      disabled={!canReply || (isRichTextEmpty(replyBody) && replyAttachments.length === 0)}
                      onClick={handleReply}
                    >
                      Send Reply
                    </Button>
                  </Group>
                </div>
              </Card>
            </div>

            <div className='support-ticket-detail-sidebar'>
              <Card className='support-ticket-panel support-ticket-sidebar-card' padding='sm'>
                <Text fw={600} mb='sm'>
                  Overview
                </Text>
                <Group gap='xs' mb='sm'>
                  <TicketStatusBadge status={detail.ticket.status} />
                  <TicketPriorityBadge priority={detail.ticket.priority} />
                </Group>

                <div className='support-ticket-sidebar-section'>
                  <Text size='xs' c='dimmed' className='support-ticket-sidebar-section-title'>
                    Ticket
                  </Text>
                  <Stack gap={8}>
                    {detail.ticket.category && (
                      <SidebarDetailRow label='Category' value={detail.ticket.category.name} />
                    )}
                    <SidebarDetailRow
                      label='Last Reply'
                      value={formatTicketDateTime(detail.ticket.lastReplyAt ?? detail.ticket.created)}
                    />
                    {detail.ticket.lastReplyByType && (
                      <SidebarDetailRow
                        label='Waiting On'
                        value={humanizeTicketActor(detail.ticket.lastReplyByType === 'staff' ? 'client' : 'staff')}
                      />
                    )}
                  </Stack>
                </div>

                <Divider my='sm' />

                <div className='support-ticket-sidebar-section'>
                  <Text size='xs' c='dimmed' className='support-ticket-sidebar-section-title'>
                    Linked Server
                  </Text>
                  <Stack gap={8}>
                    <SidebarDetailRow
                      label='Server'
                      value={
                        linkedServerPath
                          ? (
                              <SidebarValueLink to={linkedServerPath}>
                                {describeLinkedServer(detail.ticket.linkedServer)}
                              </SidebarValueLink>
                            )
                          : describeLinkedServer(detail.ticket.linkedServer)
                      }
                    />
                    {detail.ticket.linkedServer.currentStatus && (
                      <SidebarDetailRow
                        label='Status'
                        value={humanizeTicketStatus(detail.ticket.linkedServer.currentStatus)}
                      />
                    )}
                  </Stack>
                </div>

                {hasClientMetadata && (
                  <>
                    <Divider my='sm' />
                    <div className='support-ticket-sidebar-section'>
                      <Text size='xs' c='dimmed' className='support-ticket-sidebar-section-title'>
                        Attached Context
                      </Text>
                      <Stack gap={8}>
                        {clientMetadataEntries.map((entry) => (
                          <SidebarDetailRow key={entry.label} label={entry.label} value={entry.value} />
                        ))}
                      </Stack>
                    </div>
                  </>
                )}
              </Card>
            </div>
          </div>
        </div>
      </div>
    </AccountContentContainer>
  );
}
