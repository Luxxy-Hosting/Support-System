import type {
  Paginated,
  TicketAuditEvent,
  TicketCategory,
  TicketLinkedServer,
  TicketServerOption,
} from '../types/index.ts';

export type TicketStatusValue =
  | 'open'
  | 'pending'
  | 'answered'
  | 'waiting_on_client'
  | 'waiting_on_staff'
  | 'closed';

export type TicketPriorityValue = 'low' | 'normal' | 'high' | 'urgent';

export const ticketStatusOptions: Array<{ value: TicketStatusValue; label: string }> = [
  { value: 'open', label: 'Open' },
  { value: 'pending', label: 'Pending' },
  { value: 'answered', label: 'Answered' },
  { value: 'waiting_on_client', label: 'Waiting on Client' },
  { value: 'waiting_on_staff', label: 'Waiting on Staff' },
  { value: 'closed', label: 'Closed' },
];

export const ticketPriorityOptions: Array<{ value: TicketPriorityValue; label: string }> = [
  { value: 'low', label: 'Low' },
  { value: 'normal', label: 'Normal' },
  { value: 'high', label: 'High' },
  { value: 'urgent', label: 'Urgent' },
];

export const emptyPaginated = <T,>(): Paginated<T> => ({
  total: 0,
  perPage: 20,
  page: 1,
  data: [],
});

export const humanizeTicketStatus = (status: string | null | undefined): string => {
  const match = ticketStatusOptions.find((option) => option.value === status);
  return match?.label ?? humanizeSnakeCase(status);
};

export const humanizeTicketPriority = (priority: string | null | undefined): string => {
  const match = ticketPriorityOptions.find((option) => option.value === priority);
  return match?.label ?? humanizeSnakeCase(priority);
};

export const humanizeTicketActor = (actorType: string | null | undefined): string => {
  switch (actorType) {
    case 'client':
      return 'Client';
    case 'staff':
      return 'Staff';
    case 'system':
      return 'System';
    default:
      return humanizeSnakeCase(actorType);
  }
};

export const ticketStatusColor = (status: string | null | undefined): string => {
  switch (status) {
    case 'open':
      return 'blue';
    case 'pending':
      return 'yellow';
    case 'answered':
      return 'cyan';
    case 'waiting_on_client':
      return 'orange';
    case 'waiting_on_staff':
      return 'violet';
    case 'closed':
      return 'gray';
    default:
      return 'dark';
  }
};

export const ticketPriorityColor = (priority: string | null | undefined): string => {
  switch (priority) {
    case 'low':
      return 'gray';
    case 'normal':
      return 'blue';
    case 'high':
      return 'orange';
    case 'urgent':
      return 'red';
    default:
      return 'dark';
  }
};

export const formatTicketDateTime = (value: string | null | undefined): string => {
  if (!value) {
    return 'Never';
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString();
};

export const buildServerOptionLabel = (server: TicketServerOption): string => {
  return `${server.name} (#${server.uuidShort} • ${server.eggName})`;
};

export const describeLinkedServer = (server: TicketLinkedServer): string => {
  const primaryName = server.currentName ?? server.snapshotName;
  if (!primaryName) {
    return 'General ticket';
  }

  const identifier = server.currentUuidShort ?? server.snapshotUuidShort;
  const deletedSuffix = server.deletedAt ? ' (deleted)' : '';

  return identifier ? `${primaryName} (#${identifier})${deletedSuffix}` : `${primaryName}${deletedSuffix}`;
};

export const buildUserDisplayName = (user: { nameFirst: string; nameLast: string; username: string } | null | undefined): string => {
  if (!user) {
    return 'Unassigned';
  }

  const fullName = `${user.nameFirst} ${user.nameLast}`.trim();
  return fullName || user.username;
};

export const humanizeAuditEvent = (event: TicketAuditEvent): string => {
  const payload = event.payload || {};

  switch (event.event) {
    case 'ticket_created':
      return 'Ticket created';
    case 'reply_added':
      return payload.isInternal ? 'Internal note added' : 'Reply added';
    case 'ticket_closed':
      return 'Ticket closed';
    case 'ticket_reopened':
      return 'Ticket reopened';
    case 'ticket_reopened_by_reply':
      return 'Ticket reopened by client reply';
    case 'internal_note_added':
      return 'Internal note added';
    case 'status_changed':
      return `Status changed to ${humanizeTicketStatus(asString(payload.status))}`;
    case 'assignee_changed':
      return asString(payload.assignedUsername)
        ? `Assigned to ${asString(payload.assignedUsername)}`
        : 'Assignment cleared';
    case 'priority_changed':
      return asString(payload.priority)
        ? `Priority changed to ${humanizeTicketPriority(asString(payload.priority))}`
        : 'Priority cleared';
    case 'category_changed':
      return asString(payload.categoryName) ? `Category changed to ${asString(payload.categoryName)}` : 'Category cleared';
    case 'ticket_deleted':
      return 'Ticket soft-deleted';
    case 'linked_server_deleted':
      return 'Linked server was deleted';
    default:
      return humanizeSnakeCase(event.event);
  }
};

export const extractClientMetadata = (metadata: Record<string, unknown> | null | undefined): Record<string, unknown> => {
  if (!metadata || typeof metadata !== 'object') {
    return {};
  }

  const value = metadata.client;
  return value && typeof value === 'object' && !Array.isArray(value) ? (value as Record<string, unknown>) : {};
};

export const hasEnabledCategories = (categories: TicketCategory[]): boolean => categories.some((category) => category.enabled);

const humanizeSnakeCase = (value: string | null | undefined): string => {
  if (!value) {
    return 'Unknown';
  }

  return value
    .split('_')
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(' ');
};

const asString = (value: unknown): string | null => (typeof value === 'string' && value.length ? value : null);
