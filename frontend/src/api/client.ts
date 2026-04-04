import { axiosInstance } from '@/api/axios.ts';
import type {
  AdminTicketBootstrap,
  AdminTicketSettingsDetail,
  ClientTicketBootstrap,
  Paginated,
  TicketCategory,
  TicketDetail,
  TicketSummary,
} from '../types/index.ts';

export interface ClientTicketListParams {
  page: number;
  perPage: number;
  search?: string;
  status?: string;
}

export interface AdminTicketListParams {
  page: number;
  perPage: number;
  search?: string;
  status?: string;
  categoryUuid?: string;
  assignedUserUuid?: string;
  client?: string;
  server?: string;
  priority?: string;
}

export const getClientBootstrap = async (): Promise<ClientTicketBootstrap> => {
  const { data } = await axiosInstance.get('/api/client/support/bootstrap');
  return data.support;
};

export const getClientTickets = async (params: ClientTicketListParams): Promise<Paginated<TicketSummary>> => {
  const { data } = await axiosInstance.get('/api/client/support/tickets', {
    params: {
      page: params.page,
      per_page: params.perPage,
      search: params.search || undefined,
      status: params.status || undefined,
    },
  });

  return data.tickets;
};

export const createClientTicket = async (payload: {
  serverUuid?: string;
  categoryUuid?: string;
  subject: string;
  message: string;
  metadata?: Record<string, unknown>;
}): Promise<TicketDetail> => {
  const { data } = await axiosInstance.post('/api/client/support/tickets', {
    serverUuid: payload.serverUuid || undefined,
    categoryUuid: payload.categoryUuid || undefined,
    subject: payload.subject,
    message: payload.message,
    metadata: payload.metadata,
  });

  return data.ticket;
};

export const createClientTicketUpload = async (payload: {
  serverUuid?: string;
  categoryUuid?: string;
  subject: string;
  message: string;
  metadata?: Record<string, unknown>;
  files: File[];
}): Promise<TicketDetail> => {
  const form = new FormData();
  form.append('subject', payload.subject);
  form.append('message', payload.message);

  if (payload.serverUuid) {
    form.append('serverUuid', payload.serverUuid);
  }

  if (payload.categoryUuid) {
    form.append('categoryUuid', payload.categoryUuid);
  }

  if (payload.metadata) {
    form.append('metadata', JSON.stringify(payload.metadata));
  }

  for (const file of payload.files) {
    form.append('files', file, file.name);
  }

  const { data } = await axiosInstance.post('/api/client/support/tickets/upload', form, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });

  return data.ticket;
};

export const getClientTicket = async (ticketUuid: string): Promise<TicketDetail> => {
  const { data } = await axiosInstance.get(`/api/client/support/tickets/${ticketUuid}`);
  return data.ticket;
};

export const addClientReply = async (ticketUuid: string, body: string): Promise<TicketDetail> => {
  const { data } = await axiosInstance.post(`/api/client/support/tickets/${ticketUuid}/messages`, { body });
  return data.ticket;
};

export const addClientReplyUpload = async (
  ticketUuid: string,
  payload: { body: string; files: File[] },
): Promise<TicketDetail> => {
  const form = new FormData();
  form.append('body', payload.body);

  for (const file of payload.files) {
    form.append('files', file, file.name);
  }

  const { data } = await axiosInstance.post(`/api/client/support/tickets/${ticketUuid}/messages/upload`, form, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });

  return data.ticket;
};

export const updateClientTicketStatus = async (ticketUuid: string, status: string): Promise<TicketDetail> => {
  const { data } = await axiosInstance.patch(`/api/client/support/tickets/${ticketUuid}/status`, { status });
  return data.ticket;
};

export const getAdminBootstrap = async (): Promise<AdminTicketBootstrap> => {
  const { data } = await axiosInstance.get('/api/admin/support/bootstrap');
  return data.support;
};

export const getAdminSettingsDetail = async (): Promise<AdminTicketSettingsDetail> => {
  const { data } = await axiosInstance.get('/api/admin/support/settings');
  return data.settings;
};

export const getAdminTickets = async (params: AdminTicketListParams): Promise<Paginated<TicketSummary>> => {
  const { data } = await axiosInstance.get('/api/admin/support/tickets', {
    params: {
      page: params.page,
      per_page: params.perPage,
      search: params.search || undefined,
      status: params.status || undefined,
      category_uuid: params.categoryUuid || undefined,
      assigned_user_uuid: params.assignedUserUuid || undefined,
      client: params.client || undefined,
      server: params.server || undefined,
      priority: params.priority || undefined,
    },
  });

  return data.tickets;
};

export const getAdminTicket = async (ticketUuid: string): Promise<TicketDetail> => {
  const { data } = await axiosInstance.get(`/api/admin/support/tickets/${ticketUuid}`);
  return data.ticket;
};

export const addAdminMessage = async (ticketUuid: string, body: string, isInternal: boolean): Promise<TicketDetail> => {
  const { data } = await axiosInstance.post(`/api/admin/support/tickets/${ticketUuid}/messages`, {
    body,
    isInternal,
  });

  return data.ticket;
};

export const addAdminMessageUpload = async (
  ticketUuid: string,
  payload: { body: string; isInternal: boolean; files: File[] },
): Promise<TicketDetail> => {
  const form = new FormData();
  form.append('body', payload.body);
  form.append('isInternal', String(payload.isInternal));

  for (const file of payload.files) {
    form.append('files', file, file.name);
  }

  const { data } = await axiosInstance.post(`/api/admin/support/tickets/${ticketUuid}/messages/upload`, form, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });

  return data.ticket;
};

export const updateAdminTicketStatus = async (ticketUuid: string, status: string): Promise<TicketDetail> => {
  const { data } = await axiosInstance.patch(`/api/admin/support/tickets/${ticketUuid}/status`, { status });
  return data.ticket;
};

export const assignAdminTicket = async (
  ticketUuid: string,
  assignedUserUuid: string | null,
): Promise<TicketDetail> => {
  const { data } = await axiosInstance.patch(`/api/admin/support/tickets/${ticketUuid}/assignee`, {
    assignedUserUuid,
  });

  return data.ticket;
};

export const updateAdminTicketPriority = async (
  ticketUuid: string,
  priority: string | null,
): Promise<TicketDetail> => {
  const { data } = await axiosInstance.patch(`/api/admin/support/tickets/${ticketUuid}/priority`, { priority });
  return data.ticket;
};

export const updateAdminTicketCategory = async (
  ticketUuid: string,
  categoryUuid: string | null,
): Promise<TicketDetail> => {
  const { data } = await axiosInstance.patch(`/api/admin/support/tickets/${ticketUuid}/category`, { categoryUuid });
  return data.ticket;
};

export const deleteAdminTicket = async (ticketUuid: string): Promise<void> => {
  await axiosInstance.delete(`/api/admin/support/tickets/${ticketUuid}`);
};

export const updateAdminSettings = async (payload: {
  categoriesEnabled: boolean;
  allowClientClose: boolean;
  allowReplyOnClosed: boolean;
  discordWebhookEnabled: boolean;
  discordWebhookUrl?: string | null;
  discordNotifyOnTicketCreated: boolean;
  discordNotifyOnClientReply: boolean;
  discordNotifyOnStaffReply: boolean;
  discordNotifyOnInternalNote: boolean;
  discordNotifyOnStatusChange: boolean;
  discordNotifyOnAssignmentChange: boolean;
  discordNotifyOnTicketDeleted: boolean;
}): Promise<AdminTicketSettingsDetail> => {
  const { data } = await axiosInstance.put('/api/admin/support/settings', payload);
  return data.settings;
};

export const upsertAdminCategory = async (payload: {
  uuid?: string;
  name: string;
  description?: string;
  color?: string;
  sortOrder: number;
  enabled: boolean;
}): Promise<TicketCategory> => {
  const { data } = await axiosInstance.put('/api/admin/support/categories', {
    uuid: payload.uuid || undefined,
    name: payload.name,
    description: payload.description || undefined,
    color: payload.color || undefined,
    sortOrder: payload.sortOrder,
    enabled: payload.enabled,
  });

  return data.category;
};

export const deleteAdminCategory = async (categoryUuid: string): Promise<void> => {
  await axiosInstance.delete(`/api/admin/support/categories/${categoryUuid}`);
};
