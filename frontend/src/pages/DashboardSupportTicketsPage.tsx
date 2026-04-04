import { Group, Select as MantineSelect, Stack, Text, Title } from '@mantine/core';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import AccountContentContainer from '@/elements/containers/AccountContentContainer.tsx';
import ScreenBlock from '@/elements/ScreenBlock.tsx';
import Spinner from '@/elements/Spinner.tsx';
import Table, { TableData, TableRow } from '@/elements/Table.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import { useToast } from '@/providers/ToastProvider.tsx';
import { getClientBootstrap, getClientTickets } from '../api/client.ts';
import TicketPriorityBadge from '../components/TicketPriorityBadge.tsx';
import TicketStatusBadge from '../components/TicketStatusBadge.tsx';
import { describeLinkedServer, emptyPaginated, formatTicketDateTime, ticketStatusOptions } from '../helpers/tickets.ts';
import type { Paginated, TicketSummary } from '../types/index.ts';

export default function DashboardSupportTicketsPage() {
  const navigate = useNavigate();
  const { addToast } = useToast();

  const [tickets, setTickets] = useState<Paginated<TicketSummary>>(emptyPaginated());
  const [loadingBootstrap, setLoadingBootstrap] = useState(true);
  const [loadingTickets, setLoadingTickets] = useState(true);
  const [fatalError, setFatalError] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [status, setStatus] = useState<string | null>(null);
  const [page, setPage] = useState(1);

  useEffect(() => {
    let mounted = true;

    getClientBootstrap()
      .then(() => {
        if (!mounted) return;
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
      getClientTickets({
        page,
        perPage: 20,
        search: search.trim() || undefined,
        status: status || undefined,
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
  }, [addToast, page, search, status]);

  if (loadingBootstrap) {
    return (
      <AccountContentContainer title='Support'>
        <Spinner.Centered />
      </AccountContentContainer>
    );
  }

  if (fatalError) {
    return (
      <AccountContentContainer title='Support'>
        <ScreenBlock title='Support Unavailable' content={fatalError} />
      </AccountContentContainer>
    );
  }

  return (
    <AccountContentContainer title='Support'>
      <Group justify='space-between' align='end' mb='md'>
        <div>
          <Title order={1} c='white'>
            Support
          </Title>
          <Text c='dimmed'>
            Open tickets, follow replies, and create new hosting support requests.
          </Text>
        </div>
        <Button color='blue' onClick={() => navigate('/account/support/new')}>
          Create Ticket
        </Button>
      </Group>

      <Stack gap='md'>
        <div className='grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1fr)_220px]'>
          <TextInput
            label='Search Tickets'
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
        </div>

        <Table
          columns={['Subject', 'Status', 'Priority', 'Linked Server', 'Last Reply', '']}
          loading={loadingTickets}
          pagination={tickets}
          onPageSelect={setPage}
        >
          {tickets.data.map((ticket) => (
            <TableRow
              key={ticket.uuid}
              className='support-ticket-row'
              onClick={() => navigate(`/account/support/${ticket.uuid}`)}
            >
              <TableData>
                <Stack gap={4}>
                  <Text fw={600}>{ticket.subject}</Text>
                  <Text size='sm' c='dimmed'>
                    Opened by @{ticket.creator.username}
                  </Text>
                </Stack>
              </TableData>
              <TableData>
                <TicketStatusBadge status={ticket.status} />
              </TableData>
              <TableData>
                <TicketPriorityBadge priority={ticket.priority} />
              </TableData>
              <TableData>
                <Text>{describeLinkedServer(ticket.linkedServer)}</Text>
              </TableData>
              <TableData>
                <Text size='sm'>{formatTicketDateTime(ticket.lastReplyAt ?? ticket.created)}</Text>
              </TableData>
              <TableData>
                <Button variant='light' color='gray' onClick={(event) => {
                  event.stopPropagation();
                  navigate(`/account/support/${ticket.uuid}`);
                }}>
                  Open
                </Button>
              </TableData>
            </TableRow>
          ))}
        </Table>
      </Stack>
    </AccountContentContainer>
  );
}
