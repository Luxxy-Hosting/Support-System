import { Stack, Text } from '@mantine/core';
import Card from '@/elements/Card.tsx';
import { describeLinkedServer, humanizeTicketStatus } from '../helpers/tickets.ts';
import type { TicketLinkedServer } from '../types/index.ts';

export default function TicketLinkedServerCard({
  linkedServer,
  className,
}: {
  linkedServer: TicketLinkedServer;
  className?: string;
}) {
  return (
    <Card className={className}>
      <Text fw={600} mb='xs'>
        Linked Server
      </Text>
      <Stack gap={4}>
        <Text>{describeLinkedServer(linkedServer)}</Text>
        {linkedServer.currentStatus && (
          <Text size='sm' c='dimmed'>
            Status: {humanizeTicketStatus(linkedServer.currentStatus)}
          </Text>
        )}
        {linkedServer.currentOwnerUsername && (
          <Text size='sm' c='dimmed'>
            Owner: @{linkedServer.currentOwnerUsername}
          </Text>
        )}
        {linkedServer.deletedAt && (
          <Text size='sm' c='red'>
            This ticket keeps a snapshot because the linked server has been deleted.
          </Text>
        )}
        {!linkedServer.currentName && !linkedServer.snapshotName && (
          <Text size='sm' c='dimmed'>
            This is a general ticket not attached to a server.
          </Text>
        )}
      </Stack>
    </Card>
  );
}
