import { Avatar, Group, Stack, Text } from '@mantine/core';
import Card from '@/elements/Card.tsx';
import Badge from '@/elements/Badge.tsx';
import { formatTicketDateTime, humanizeTicketActor } from '../helpers/tickets.ts';
import type { TicketMessage } from '../types/index.ts';
import SupportMessageAttachments from './SupportMessageAttachments.tsx';
import SupportRichTextContent from './SupportRichTextContent.tsx';

interface Props {
  messages: TicketMessage[];
  emptyText?: string;
  scrollable?: boolean;
}

const avatarInitials = (message: TicketMessage): string => {
  const source = (message.authorDisplayName || message.authorUsername || humanizeTicketActor(message.authorType)).trim();
  const parts = source.split(/\s+/).filter(Boolean);

  if (!parts.length) {
    return '?';
  }

  if (parts.length === 1) {
    return parts[0].slice(0, 2).toUpperCase();
  }

  return `${parts[0][0] ?? ''}${parts[1][0] ?? ''}`.toUpperCase();
};

const avatarColor = (message: TicketMessage): string => {
  if (message.isInternal) {
    return 'amber';
  }

  switch (message.authorType) {
    case 'staff':
      return 'blue';
    case 'client':
      return 'violet';
    case 'system':
      return 'gray';
    default:
      return 'dark';
  }
};

export default function TicketConversation({
  messages,
  emptyText = 'No messages yet.',
  scrollable = false,
}: Props) {
  if (!messages.length) {
    return <Text c='dimmed'>{emptyText}</Text>;
  }

  const content = (
    <Stack gap='md'>
      {messages.map((message) => (
        <Card
          key={message.uuid}
          className={`support-ticket-message-card ${message.isInternal ? 'border-amber-500/35!' : ''}`.trim()}
          leftStripeClassName={message.isInternal ? 'bg-amber-500/70' : message.authorType === 'staff' ? 'bg-blue-500/60' : 'bg-violet-500/60'}
        >
          <div className='support-ticket-message-shell'>
            <Avatar
              size='md'
              radius='xl'
              color={avatarColor(message)}
              variant='light'
              src={message.authorAvatar || undefined}
              className='support-ticket-message-avatar'
            >
              {avatarInitials(message)}
            </Avatar>

            <div className='support-ticket-message-content'>
              <div className='support-ticket-message-header'>
                <div className='support-ticket-message-author'>
                  <Group gap='xs' wrap='wrap'>
                    <Text fw={600} size='sm'>
                      {message.authorDisplayName || message.authorUsername}
                    </Text>
                    {message.isInternal && (
                      <Badge color='amber' variant='light'>
                        Internal Note
                      </Badge>
                    )}
                  </Group>
                  <Text size='xs' c='dimmed'>
                    {humanizeTicketActor(message.authorType)} • @{message.authorUsername}
                  </Text>
                </div>

                <Text size='xs' c='dimmed' className='support-ticket-message-time'>
                  {formatTicketDateTime(message.created)}
                </Text>
              </div>

              {message.body.trim().length > 0 && <SupportRichTextContent value={message.body} />}
              {message.attachments.length > 0 && <SupportMessageAttachments attachments={message.attachments} />}
            </div>
          </div>
        </Card>
      ))}
    </Stack>
  );

  if (!scrollable) {
    return content;
  }

  return <div className='support-ticket-conversation-scroll'>{content}</div>;
}
