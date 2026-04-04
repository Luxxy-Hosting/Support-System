import Badge from '@/elements/Badge.tsx';
import { humanizeTicketPriority, ticketPriorityColor } from '../helpers/tickets.ts';

export default function TicketPriorityBadge({ priority }: { priority: string | null | undefined }) {
  if (!priority) {
    return null;
  }

  return (
    <Badge color={ticketPriorityColor(priority)} variant='light'>
      {humanizeTicketPriority(priority)}
    </Badge>
  );
}
