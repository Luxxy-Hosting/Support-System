export interface Paginated<T> {
  total: number;
  perPage: number;
  page: number;
  data: T[];
}

export interface TicketSettings {
  uuid: string;
  categoriesEnabled: boolean;
  allowClientClose: boolean;
  allowReplyOnClosed: boolean;
  created: string;
  updated: string;
}

export interface DiscordWebhookSettings {
  enabled: boolean;
  webhookUrl: string | null;
  notifyOnTicketCreated: boolean;
  notifyOnClientReply: boolean;
  notifyOnStaffReply: boolean;
  notifyOnInternalNote: boolean;
  notifyOnStatusChange: boolean;
  notifyOnAssignmentChange: boolean;
  notifyOnTicketDeleted: boolean;
}

export interface TicketCategory {
  uuid: string;
  name: string;
  description: string | null;
  color: string | null;
  sortOrder: number;
  enabled: boolean;
  created: string;
  updated: string;
}

export interface TicketCategorySummary {
  uuid: string;
  name: string;
  color: string | null;
}

export interface TicketUserSummary {
  uuid: string;
  username: string;
  nameFirst: string;
  nameLast: string;
  admin: boolean;
}

export interface TicketLinkedServer {
  uuid: string | null;
  snapshotName: string | null;
  snapshotUuidShort: number | null;
  deletedAt: string | null;
  currentName: string | null;
  currentUuidShort: number | null;
  currentStatus: string | null;
  currentIsSuspended: boolean | null;
  currentOwnerUsername: string | null;
}

export interface TicketServerOption {
  uuid: string;
  uuidShort: number;
  name: string;
  ownerUsername: string;
  nestName: string;
  eggName: string;
  isSuspended: boolean;
  status: string | null;
}

export interface TicketAttachment {
  uuid: string;
  originalName: string;
  contentType: string;
  mediaType: 'image' | 'video';
  size: number;
  url: string;
  created: string;
}

export interface TicketMessage {
  uuid: string;
  authorUserUuid: string | null;
  authorUsername: string;
  authorDisplayName: string;
  authorAvatar: string | null;
  authorType: string;
  body: string;
  isInternal: boolean;
  attachments: TicketAttachment[];
  created: string;
  updated: string;
}

export interface TicketAuditEvent {
  uuid: string;
  actorUserUuid: string | null;
  actorUsername: string | null;
  actorType: string;
  event: string;
  payload: Record<string, unknown>;
  created: string;
}

export interface TicketSummary {
  uuid: string;
  subject: string;
  status: string;
  priority: string | null;
  creator: TicketUserSummary;
  category: TicketCategorySummary | null;
  assignedUser: TicketUserSummary | null;
  linkedServer: TicketLinkedServer;
  lastReplyAt: string | null;
  lastReplyByType: string | null;
  created: string;
  updated: string;
  closedAt: string | null;
}

export interface TicketDetail {
  ticket: TicketSummary;
  metadata: Record<string, unknown>;
  messages: TicketMessage[];
  auditEvents: TicketAuditEvent[];
}

export interface ClientTicketBootstrap {
  settings: TicketSettings;
  categories: TicketCategory[];
  servers: TicketServerOption[];
}

export interface AdminTicketBootstrap {
  settings: TicketSettings;
  categories: TicketCategory[];
  staffUsers: TicketUserSummary[];
}

export interface AdminTicketSettingsDetail {
  settings: TicketSettings;
  discordWebhook: DiscordWebhookSettings;
}
