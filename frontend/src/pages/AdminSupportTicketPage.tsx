import { Divider, Group, Select as MantineSelect, Stack, Text, Title } from '@mantine/core';
import { type ReactNode, useEffect, useMemo, useState } from 'react';
import { NavLink, useNavigate, useParams } from 'react-router';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import AdminContentContainer from '@/elements/containers/AdminContentContainer.tsx';
import ScreenBlock from '@/elements/ScreenBlock.tsx';
import Spinner from '@/elements/Spinner.tsx';
import ConfirmationModal from '@/elements/modals/ConfirmationModal.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import {
  addAdminMessage,
  addAdminMessageUpload,
  assignAdminTicket,
  deleteAdminTicket,
  getAdminBootstrap,
  getAdminTicket,
  updateAdminTicketCategory,
  updateAdminTicketPriority,
  updateAdminTicketStatus,
} from '../api/client.ts';
import SupportAttachmentPicker from '../components/SupportAttachmentPicker.tsx';
import TicketConversation from '../components/TicketConversation.tsx';
import TicketPriorityBadge from '../components/TicketPriorityBadge.tsx';
import TicketStatusBadge from '../components/TicketStatusBadge.tsx';
import SupportRichTextEditor from '../components/SupportRichTextEditor.tsx';
import {
  buildUserDisplayName,
  describeLinkedServer,
  extractClientMetadata,
  formatTicketDateTime,
  humanizeAuditEvent,
  humanizeTicketStatus,
  ticketPriorityOptions,
  ticketStatusOptions,
} from '../helpers/tickets.ts';
import { isRichTextEmpty } from '../helpers/richText.ts';
import type { AdminTicketBootstrap, TicketDetail } from '../types/index.ts';

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

export default function AdminSupportTicketPage() {
  const navigate = useNavigate();
  const { ticket: ticketUuid } = useParams();
  const { addToast } = useToast();

  const canReplyAll = useAdminCan('tickets.reply-all');
  const canChangeStatus = useAdminCan('tickets.change-status');
  const canAssign = useAdminCan('tickets.assign');
  const canAddInternalNotes = useAdminCan('tickets.add-internal-notes');
  const canDelete = useAdminCan('tickets.delete');

  const [bootstrap, setBootstrap] = useState<AdminTicketBootstrap | null>(null);
  const [detail, setDetail] = useState<TicketDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [fatalError, setFatalError] = useState<string | null>(null);
  const [messageBody, setMessageBody] = useState('');
  const [messageAttachments, setMessageAttachments] = useState<File[]>([]);
  const [composerLoading, setComposerLoading] = useState(false);
  const [controlLoading, setControlLoading] = useState(false);
  const [savingControl, setSavingControl] = useState<'status' | 'priority' | 'category' | 'assignee' | null>(null);
  const [deleteOpen, setDeleteOpen] = useState(false);

  const [statusValue, setStatusValue] = useState<string | null>(null);
  const [priorityValue, setPriorityValue] = useState<string | null>(null);
  const [categoryValue, setCategoryValue] = useState<string | null>(null);
  const [assigneeValue, setAssigneeValue] = useState<string | null>(null);

  useEffect(() => {
    if (!ticketUuid) {
      setFatalError('Ticket not found.');
      setLoading(false);
      return;
    }

    let mounted = true;
    setLoading(true);

    Promise.all([getAdminBootstrap(), getAdminTicket(ticketUuid)])
      .then(([bootstrapResponse, detailResponse]) => {
        if (!mounted) return;
        setBootstrap(bootstrapResponse);
        setDetail(detailResponse);
        setStatusValue(detailResponse.ticket.status);
        setPriorityValue(detailResponse.ticket.priority);
        setCategoryValue(detailResponse.ticket.category?.uuid ?? null);
        setAssigneeValue(detailResponse.ticket.assignedUser?.uuid ?? null);
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

  const categoryOptions = useMemo(
    () =>
      (bootstrap?.categories ?? []).map((category) => ({
        value: category.uuid,
        label: category.name,
      })),
    [bootstrap?.categories],
  );

  const staffOptions = useMemo(
    () =>
      (bootstrap?.staffUsers ?? []).map((user) => ({
        value: user.uuid,
        label: buildUserDisplayName(user),
      })),
    [bootstrap?.staffUsers],
  );

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
  const linkedAdminServerPath = detail?.ticket.linkedServer.uuid
    ? `/admin/servers/${detail.ticket.linkedServer.uuid}`
    : null;
  const creatorAdminUserPath = detail ? `/admin/users/${detail.ticket.creator.uuid}` : null;
  const assignedAdminUserPath = detail?.ticket.assignedUser
    ? `/admin/users/${detail.ticket.assignedUser.uuid}`
    : null;

  const syncDetail = (nextTicket: TicketDetail, options?: { preserveMessageBody?: boolean }) => {
    setDetail(nextTicket);
    setStatusValue(nextTicket.ticket.status);
    setPriorityValue(nextTicket.ticket.priority);
    setCategoryValue(nextTicket.ticket.category?.uuid ?? null);
    setAssigneeValue(nextTicket.ticket.assignedUser?.uuid ?? null);
    if (!options?.preserveMessageBody) {
      setMessageBody('');
      setMessageAttachments([]);
    }
  };

  const handleAdminMessage = async (isInternal: boolean) => {
    if (!ticketUuid || (isRichTextEmpty(messageBody) && messageAttachments.length === 0)) {
      return;
    }

    try {
      setComposerLoading(true);
      const nextTicket = messageAttachments.length > 0
        ? await addAdminMessageUpload(ticketUuid, {
            body: isRichTextEmpty(messageBody) ? '' : messageBody,
            isInternal,
            files: messageAttachments,
          })
        : await addAdminMessage(ticketUuid, messageBody, isInternal);
      syncDetail(nextTicket);
      addToast(isInternal ? 'Internal note saved.' : 'Reply sent to client.', 'success');
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setComposerLoading(false);
    }
  };

  const handleStatusChange = async (nextValue: string | null) => {
    if (!ticketUuid || !nextValue || nextValue === statusValue) {
      return;
    }

    try {
      setStatusValue(nextValue);
      setSavingControl('status');
      const nextTicket = await updateAdminTicketStatus(ticketUuid, nextValue);
      syncDetail(nextTicket, { preserveMessageBody: true });
      addToast('Ticket status updated.', 'success');
    } catch (error) {
      setStatusValue(detail.ticket.status);
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setSavingControl(null);
    }
  };

  const handlePriorityChange = async (nextValue: string | null) => {
    if (!ticketUuid || nextValue === priorityValue) {
      return;
    }

    try {
      setPriorityValue(nextValue);
      setSavingControl('priority');
      const nextTicket = await updateAdminTicketPriority(ticketUuid, nextValue ?? null);
      syncDetail(nextTicket, { preserveMessageBody: true });
      addToast('Ticket priority updated.', 'success');
    } catch (error) {
      setPriorityValue(detail.ticket.priority);
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setSavingControl(null);
    }
  };

  const handleCategoryChange = async (nextValue: string | null) => {
    if (!ticketUuid || nextValue === categoryValue) {
      return;
    }

    try {
      setCategoryValue(nextValue);
      setSavingControl('category');
      const nextTicket = await updateAdminTicketCategory(ticketUuid, nextValue ?? null);
      syncDetail(nextTicket, { preserveMessageBody: true });
      addToast('Ticket category updated.', 'success');
    } catch (error) {
      setCategoryValue(detail.ticket.category?.uuid ?? null);
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setSavingControl(null);
    }
  };

  const handleAssigneeChange = async (nextValue: string | null) => {
    if (!ticketUuid || nextValue === assigneeValue) {
      return;
    }

    try {
      setAssigneeValue(nextValue);
      setSavingControl('assignee');
      const nextTicket = await assignAdminTicket(ticketUuid, nextValue ?? null);
      syncDetail(nextTicket, { preserveMessageBody: true });
      addToast('Ticket assignment updated.', 'success');
    } catch (error) {
      setAssigneeValue(detail.ticket.assignedUser?.uuid ?? null);
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setSavingControl(null);
    }
  };

  const handleDelete = async () => {
    if (!ticketUuid) {
      return;
    }

    try {
      setControlLoading(true);
      await deleteAdminTicket(ticketUuid);
      addToast('Ticket deleted.', 'success');
      navigate('/admin/support');
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setControlLoading(false);
      setDeleteOpen(false);
    }
  };

  if (loading) {
    return (
      <AdminContentContainer title='Ticket'>
        <Spinner.Centered />
      </AdminContentContainer>
    );
  }

  if (!detail || !bootstrap || fatalError) {
    return (
      <AdminContentContainer title='Ticket'>
        <ScreenBlock title='Ticket Unavailable' content={fatalError ?? 'Unable to load ticket details.'} />
      </AdminContentContainer>
    );
  }

  return (
    <AdminContentContainer title='Support Ticket'>
      <ConfirmationModal
        opened={deleteOpen}
        onClose={() => setDeleteOpen(false)}
        title='Delete Ticket'
        confirm='Delete Ticket'
        onConfirmed={handleDelete}
      >
        <Text size='sm'>This performs a soft delete so history remains in the database for auditability.</Text>
      </ConfirmationModal>

      <div className='support-ticket-page-header'>
        <div className='support-ticket-page-heading'>
          <Title order={1} c='white'>
            {detail.ticket.subject}
          </Title>
          <Text c='dimmed'>
            Client @{detail.ticket.creator.username} • created {formatTicketDateTime(detail.ticket.created)}
          </Text>
        </div>
        <div className='support-ticket-toolbar'>
          {canChangeStatus && (
            <>
              <div className='support-ticket-toolbar-field'>
                <Text size='xs' c='dimmed' className='support-ticket-toolbar-label'>
                  Status
                </Text>
                <MantineSelect
                  size='sm'
                  data={ticketStatusOptions}
                  value={statusValue}
                  onChange={(value) => void handleStatusChange(value)}
                  disabled={savingControl !== null}
                />
              </div>
              <div className='support-ticket-toolbar-field'>
                <Text size='xs' c='dimmed' className='support-ticket-toolbar-label'>
                  Priority
                </Text>
                <MantineSelect
                  size='sm'
                  data={ticketPriorityOptions}
                  value={priorityValue}
                  allowDeselect
                  clearable
                  onChange={(value) => void handlePriorityChange(value)}
                  disabled={savingControl !== null}
                />
              </div>
              <div className='support-ticket-toolbar-field'>
                <Text size='xs' c='dimmed' className='support-ticket-toolbar-label'>
                  Category
                </Text>
                <MantineSelect
                  size='sm'
                  data={categoryOptions}
                  value={categoryValue}
                  allowDeselect
                  clearable
                  searchable
                  onChange={(value) => void handleCategoryChange(value)}
                  disabled={savingControl !== null}
                />
              </div>
            </>
          )}

          {canAssign && (
            <div className='support-ticket-toolbar-field support-ticket-toolbar-field-wide'>
              <Text size='xs' c='dimmed' className='support-ticket-toolbar-label'>
                Assigned Staff
              </Text>
              <MantineSelect
                size='sm'
                data={staffOptions}
                value={assigneeValue}
                allowDeselect
                clearable
                searchable
                onChange={(value) => void handleAssigneeChange(value)}
                disabled={savingControl !== null}
              />
            </div>
          )}

          <div className='support-ticket-toolbar-buttons'>
            {savingControl && (
              <Text size='xs' c='dimmed' className='support-ticket-toolbar-saving'>
                Saving {savingControl}...
              </Text>
            )}
            <Button variant='light' color='gray' onClick={() => navigate('/admin/support')}>
              Back to Queue
            </Button>
            {canDelete && (
              <Button color='red' variant='light' onClick={() => setDeleteOpen(true)}>
                Delete Ticket
              </Button>
            )}
          </div>
        </div>
      </div>

      <div className='support-ticket-detail-page'>
        <div className='support-ticket-workspace'>
          <div className='support-ticket-detail-grid'>
            <div className='support-ticket-detail-main'>
              <Card className='support-ticket-panel support-ticket-workspace-card'>
                <Text fw={600} mb='md'>
                  Conversation
                </Text>
                <div className='support-ticket-thread-body'>
                  <TicketConversation
                    messages={detail.messages}
                    emptyText='No replies or internal notes have been added yet.'
                    scrollable
                  />
                </div>
                <div className='support-ticket-composer-section'>
                  <Divider mb='md' />
                  <SupportRichTextEditor
                    value={messageBody}
                    onChange={setMessageBody}
                    placeholder='Write a reply or internal note...'
                  />
                  <SupportAttachmentPicker
                    files={messageAttachments}
                    disabled={composerLoading}
                    onChange={setMessageAttachments}
                  />
                  <Group justify='flex-end' mt='md'>
                    {canAddInternalNotes && (
                      <Button
                        variant='default'
                        loading={composerLoading}
                        disabled={isRichTextEmpty(messageBody) && messageAttachments.length === 0}
                        onClick={() => handleAdminMessage(true)}
                      >
                        Add Internal Note
                      </Button>
                    )}
                    <Button
                      color='blue'
                      loading={composerLoading}
                      disabled={!canReplyAll || (isRichTextEmpty(messageBody) && messageAttachments.length === 0)}
                      onClick={() => handleAdminMessage(false)}
                    >
                      Reply to Client
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
                    <SidebarDetailRow
                      label='Category'
                      value={detail.ticket.category?.name ?? 'Uncategorized'}
                    />
                    <SidebarDetailRow
                      label='Assigned To'
                      value={
                        assignedAdminUserPath && detail.ticket.assignedUser
                          ? (
                              <SidebarValueLink to={assignedAdminUserPath}>
                                {buildUserDisplayName(detail.ticket.assignedUser)}
                              </SidebarValueLink>
                            )
                          : buildUserDisplayName(detail.ticket.assignedUser)
                      }
                    />
                    <SidebarDetailRow
                      label='Last Reply'
                      value={formatTicketDateTime(detail.ticket.lastReplyAt ?? detail.ticket.created)}
                    />
                    {detail.ticket.closedAt && (
                      <SidebarDetailRow
                        label='Closed'
                        value={formatTicketDateTime(detail.ticket.closedAt)}
                      />
                    )}
                  </Stack>
                </div>

                <Divider my='sm' />

                <div className='support-ticket-sidebar-section'>
                  <Text size='xs' c='dimmed' className='support-ticket-sidebar-section-title'>
                    Server & Client
                  </Text>
                  <Stack gap={8}>
                    <SidebarDetailRow
                      label='Linked Server'
                      value={
                        linkedAdminServerPath
                          ? (
                              <SidebarValueLink to={linkedAdminServerPath}>
                                {describeLinkedServer(detail.ticket.linkedServer)}
                              </SidebarValueLink>
                            )
                          : describeLinkedServer(detail.ticket.linkedServer)
                      }
                    />
                    {detail.ticket.linkedServer.currentStatus && (
                      <SidebarDetailRow
                        label='Server Status'
                        value={humanizeTicketStatus(detail.ticket.linkedServer.currentStatus)}
                      />
                    )}
                    <SidebarDetailRow
                      label='Client'
                      value={
                        creatorAdminUserPath
                          ? (
                              <SidebarValueLink to={creatorAdminUserPath}>
                                {`${detail.ticket.creator.nameFirst} ${detail.ticket.creator.nameLast}`}
                              </SidebarValueLink>
                            )
                          : `${detail.ticket.creator.nameFirst} ${detail.ticket.creator.nameLast}`
                      }
                    />
                    <SidebarDetailRow
                      label='Username'
                      value={
                        creatorAdminUserPath
                          ? (
                              <SidebarValueLink to={creatorAdminUserPath}>
                                @{detail.ticket.creator.username}
                              </SidebarValueLink>
                            )
                          : `@${detail.ticket.creator.username}`
                      }
                    />
                  </Stack>
                </div>

                {detail.ticket.linkedServer.deletedAt && (
                  <Text size='sm' c='red' mt='sm' className='support-ticket-sidebar-note'>
                    This ticket keeps a snapshot because the linked server has been deleted.
                  </Text>
                )}

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

              <Card className='support-ticket-panel support-ticket-audit-panel support-ticket-sidebar-card' padding='sm'>
                <Text fw={600} mb='sm'>
                  Audit Timeline
                </Text>
                <div className='support-ticket-audit-list'>
                  {detail.auditEvents.length ? (
                    detail.auditEvents.map((event) => (
                      <div key={event.uuid} className='support-ticket-audit-entry'>
                        <Text fw={600} size='sm'>
                          {humanizeAuditEvent(event)}
                        </Text>
                        <Text size='xs' c='dimmed'>
                          {event.actorUsername ? `@${event.actorUsername}` : 'System'} • {formatTicketDateTime(event.created)}
                        </Text>
                      </div>
                    ))
                  ) : (
                    <Text c='dimmed'>No audit events recorded yet.</Text>
                  )}
                </div>
              </Card>
            </div>
          </div>
        </div>
      </div>
    </AdminContentContainer>
  );
}
