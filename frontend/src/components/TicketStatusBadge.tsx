import Badge from '@/elements/Badge.tsx';
import { humanizeTicketStatus, ticketStatusColor } from '../helpers/tickets.ts';

export default function TicketStatusBadge({ status }: { status: string }) {
  return (
    <Badge color={ticketStatusColor(status)} variant='light'>
      {humanizeTicketStatus(status)}
    </Badge>
  );
}
