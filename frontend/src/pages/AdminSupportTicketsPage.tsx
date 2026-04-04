import { Group, Select as MantineSelect, SimpleGrid, Stack, Text } from '@mantine/core';
import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import AdminContentContainer from '@/elements/containers/AdminContentContainer.tsx';
import ScreenBlock from '@/elements/ScreenBlock.tsx';
import Spinner from '@/elements/Spinner.tsx';
import Table, { TableData, TableRow } from '@/elements/Table.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { getAdminBootstrap, getAdminTickets } from '../api/client.ts';
import TicketPriorityBadge from '../components/TicketPriorityBadge.tsx';
import TicketStatusBadge from '../components/TicketStatusBadge.tsx';
import {
  buildUserDisplayName,
  describeLinkedServer,
  emptyPaginated,
  formatTicketDateTime,
  ticketPriorityOptions,
  ticketStatusOptions,
} from '../helpers/tickets.ts';
import type { AdminTicketBootstrap, Paginated, TicketSummary } from '../types/index.ts';

export default function AdminSupportTicketsPage() {
  const navigate = useNavigate();
  const { addToast } = useToast();

  const canManageSettings = useAdminCan('tickets.manage-settings');
  const canManageCategories = useAdminCan('tickets.manage-categories');

  const [bootstrap, setBootstrap] = useState<AdminTicketBootstrap | null>(null);
  const [tickets, setTickets] = useState<Paginated<TicketSummary>>(emptyPaginated());
  const [loadingBootstrap, setLoadingBootstrap] = useState(true);
  const [loadingTickets, setLoadingTickets] = useState(true);
  const [fatalError, setFatalError] = useState<string | null>(null);

  const [search, setSearch] = useState('');
  const [status, setStatus] = useState<string | null>(null);
  const [categoryUuid, setCategoryUuid] = useState<string | null>(null);
  const [assignedUserUuid, setAssignedUserUuid] = useState<string | null>(null);
  const [clientFilter, setClientFilter] = useState('');
  const [serverFilter, setServerFilter] = useState('');
  const [priority, setPriority] = useState<string | null>(null);
  const [page, setPage] = useState(1);

  useEffect(() => {
    let mounted = true;

    getAdminBootstrap()
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
          setLoadingBootstrap(false);
        }
      });

    return () => {
      mounted = false;
    };
  }, []);

  useEffect(() => {
    let mounted = true;
    setLoadingTickets(true);

    const timer = window.setTimeout(() => {
      getAdminTickets({
        page,
        perPage: 20,
        search: search.trim() || undefined,
        status: status || undefined,
        categoryUuid: categoryUuid || undefined,
        assignedUserUuid: assignedUserUuid || undefined,
        client: clientFilter.trim() || undefined,
        server: serverFilter.trim() || undefined,
        priority: priority || undefined,
      })
        .then((response) => {
          if (!mounted) return;
          setTickets(response);
        })
        .catch((error) => {
          if (!mounted) return;
          addToast(httpErrorToHuman(error), 'error');
        })
        .finally(() => {
          if (mounted) {
            setLoadingTickets(false);
          }
        });
    }, 250);

    return () => {
      mounted = false;
      window.clearTimeout(timer);
    };
  }, [addToast, assignedUserUuid, categoryUuid, clientFilter, page, priority, search, serverFilter, status]);

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

  if (loadingBootstrap) {
    return (
      <AdminContentContainer title='Support Center'>
        <Spinner.Centered />
      </AdminContentContainer>
    );
  }

  if (fatalError || !bootstrap) {
    return (
      <AdminContentContainer title='Support Center'>
        <ScreenBlock title='Support Unavailable' content={fatalError ?? 'Unable to load support admin data.'} />
      </AdminContentContainer>
    );
  }

  return (
    <AdminContentContainer title='Support Center'>
      <Stack gap='md'>
        {(canManageSettings || canManageCategories) && (
          <Group justify='flex-end'>
            <Button
              variant='light'
              color='gray'
              onClick={() => navigate('/admin/extensions/dev.luxxy.supportsystem')}
            >
              Extension Settings
            </Button>
          </Group>
        )}

        <Card>
          <Text fw={600} mb='md'>
            Queue Filters
          </Text>
          <SimpleGrid cols={{ base: 1, md: 2, xl: 3 }}>
            <TextInput
              label='Search'
              value={search}
              onChange={(event) => {
                setSearch(event.currentTarget.value);
                setPage(1);
              }}
            />
            <MantineSelect
              label='Status'
              data={ticketStatusOptions}
              value={status}
              allowDeselect
              clearable
              onChange={(value) => {
                setStatus(value);
                setPage(1);
              }}
            />
            <MantineSelect
              label='Category'
              data={categoryOptions}
              value={categoryUuid}
              allowDeselect
              clearable
              searchable
              onChange={(value) => {
                setCategoryUuid(value);
                setPage(1);
              }}
            />
            <MantineSelect
              label='Assigned Staff'
              data={staffOptions}
              value={assignedUserUuid}
              allowDeselect
              clearable
              searchable
              onChange={(value) => {
                setAssignedUserUuid(value);
                setPage(1);
              }}
            />
            <TextInput
              label='Client'
              value={clientFilter}
              onChange={(event) => {
                setClientFilter(event.currentTarget.value);
                setPage(1);
              }}
            />
            <TextInput
              label='Linked Server'
              value={serverFilter}
              onChange={(event) => {
                setServerFilter(event.currentTarget.value);
                setPage(1);
              }}
            />
            <MantineSelect
              label='Priority'
              data={ticketPriorityOptions}
              value={priority}
              allowDeselect
              clearable
              onChange={(value) => {
                setPriority(value);
                setPage(1);
              }}
            />
          </SimpleGrid>
        </Card>

        <Table
          columns={['Subject', 'Status', 'Priority', 'Client', 'Linked Server', 'Assigned', 'Last Reply', '']}
          loading={loadingTickets}
          pagination={tickets}
          onPageSelect={setPage}
        >
          {tickets.data.map((ticket) => (
            <TableRow key={ticket.uuid} className='support-ticket-row' onClick={() => navigate(`/admin/support/${ticket.uuid}`)}>
              <TableData>
                <Stack gap={4}>
                  <Text fw={600}>{ticket.subject}</Text>
                  <Text size='sm' c='dimmed'>
                    Created {formatTicketDateTime(ticket.created)}
                  </Text>
                </Stack>
              </TableData>
              <TableData>
                <TicketStatusBadge status={ticket.status} />
              </TableData>
              <TableData>
                <TicketPriorityBadge priority={ticket.priority} />
              </TableData>
              <TableData>@{ticket.creator.username}</TableData>
              <TableData>{describeLinkedServer(ticket.linkedServer)}</TableData>
              <TableData>{buildUserDisplayName(ticket.assignedUser)}</TableData>
              <TableData>{formatTicketDateTime(ticket.lastReplyAt ?? ticket.created)}</TableData>
              <TableData>
                <Button variant='light' color='gray' onClick={(event) => {
                  event.stopPropagation();
                  navigate(`/admin/support/${ticket.uuid}`);
                }}>
                  Open
                </Button>
              </TableData>
            </TableRow>
          ))}
        </Table>
      </Stack>
    </AdminContentContainer>
  );
}
