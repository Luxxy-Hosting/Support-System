import { Group, Select as MantineSelect, NumberInput, SimpleGrid, Stack, Switch, Text, Title } from '@mantine/core';
import { useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import ConfirmationModal from '@/elements/modals/ConfirmationModal.tsx';
import ScreenBlock from '@/elements/ScreenBlock.tsx';
import Spinner from '@/elements/Spinner.tsx';
import { useAdminCan } from '@/plugins/usePermissions.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import {
  deleteAdminCategory,
  getAdminBootstrap,
  getAdminSettingsDetail,
  updateAdminSettings,
  upsertAdminCategory,
} from '../api/client.ts';
import type { AdminTicketBootstrap, AdminTicketSettingsDetail } from '../types/index.ts';

const defaultSettingsForm = {
  categoriesEnabled: true,
  allowClientClose: true,
  allowReplyOnClosed: false,
  createTicketRateLimitHits: 20,
  createTicketRateLimitWindowSeconds: 300,
  maxOpenTicketsPerUser: 0,
  discordWebhookEnabled: false,
  discordWebhookUrl: '',
  discordNotifyOnTicketCreated: true,
  discordNotifyOnClientReply: true,
  discordNotifyOnStaffReply: true,
  discordNotifyOnInternalNote: false,
  discordNotifyOnStatusChange: true,
  discordNotifyOnAssignmentChange: true,
  discordNotifyOnTicketDeleted: false,
};

function buildSettingsForm(bootstrap: AdminTicketBootstrap, detail: AdminTicketSettingsDetail | null) {
  return {
    categoriesEnabled: bootstrap.settings.categoriesEnabled,
    allowClientClose: bootstrap.settings.allowClientClose,
    allowReplyOnClosed: bootstrap.settings.allowReplyOnClosed,
    createTicketRateLimitHits: bootstrap.settings.createTicketRateLimitHits,
    createTicketRateLimitWindowSeconds: bootstrap.settings.createTicketRateLimitWindowSeconds,
    maxOpenTicketsPerUser: bootstrap.settings.maxOpenTicketsPerUser,
    discordWebhookEnabled: detail?.discordWebhook.enabled ?? false,
    discordWebhookUrl: detail?.discordWebhook.webhookUrl ?? '',
    discordNotifyOnTicketCreated: detail?.discordWebhook.notifyOnTicketCreated ?? true,
    discordNotifyOnClientReply: detail?.discordWebhook.notifyOnClientReply ?? true,
    discordNotifyOnStaffReply: detail?.discordWebhook.notifyOnStaffReply ?? true,
    discordNotifyOnInternalNote: detail?.discordWebhook.notifyOnInternalNote ?? false,
    discordNotifyOnStatusChange: detail?.discordWebhook.notifyOnStatusChange ?? true,
    discordNotifyOnAssignmentChange: detail?.discordWebhook.notifyOnAssignmentChange ?? true,
    discordNotifyOnTicketDeleted: detail?.discordWebhook.notifyOnTicketDeleted ?? false,
  };
}

export default function AdminSupportSettingsPage() {
  const navigate = useNavigate();
  const { addToast } = useToast();

  const canManageSettings = useAdminCan('tickets.manage-settings');
  const canManageCategories = useAdminCan('tickets.manage-categories');

  const [bootstrap, setBootstrap] = useState<AdminTicketBootstrap | null>(null);
  const [loading, setLoading] = useState(true);
  const [fatalError, setFatalError] = useState<string | null>(null);

  const [settingsSaving, setSettingsSaving] = useState(false);
  const [settingsForm, setSettingsForm] = useState(defaultSettingsForm);

  const [categoryDeleteOpen, setCategoryDeleteOpen] = useState(false);
  const [categorySaving, setCategorySaving] = useState(false);
  const [selectedCategoryUuid, setSelectedCategoryUuid] = useState<string | null>(null);
  const [categoryForm, setCategoryForm] = useState({
    name: '',
    description: '',
    color: '',
    sortOrder: 0,
    enabled: true,
  });

  const load = () => {
    setLoading(true);
    setFatalError(null);
    Promise.all([getAdminBootstrap(), canManageSettings ? getAdminSettingsDetail() : Promise.resolve(null)])
      .then(([bootstrapResponse, settingsDetail]) => {
        setBootstrap(bootstrapResponse);
        setSettingsForm(buildSettingsForm(bootstrapResponse, settingsDetail));
      })
      .catch((error) => setFatalError(httpErrorToHuman(error)))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    load();
  }, [canManageSettings]);

  const categoryOptions = useMemo(
    () =>
      (bootstrap?.categories ?? []).map((category) => ({
        value: category.uuid,
        label: category.name,
      })),
    [bootstrap?.categories],
  );

  const enabledCategoryCount = useMemo(
    () => (bootstrap?.categories ?? []).filter((category) => category.enabled).length,
    [bootstrap?.categories],
  );

  useEffect(() => {
    if (!selectedCategoryUuid || !bootstrap) {
      setCategoryForm({
        name: '',
        description: '',
        color: '',
        sortOrder: 0,
        enabled: true,
      });
      return;
    }

    const category = bootstrap.categories.find((entry) => entry.uuid === selectedCategoryUuid);
    if (!category) {
      return;
    }

    setCategoryForm({
      name: category.name,
      description: category.description ?? '',
      color: category.color ?? '',
      sortOrder: category.sortOrder,
      enabled: category.enabled,
    });
  }, [bootstrap, selectedCategoryUuid]);

  const refreshBootstrap = async () => {
    const response = await getAdminBootstrap();
    setBootstrap(response);
    setSettingsForm((current) => ({
      ...current,
      categoriesEnabled: response.settings.categoriesEnabled,
      allowClientClose: response.settings.allowClientClose,
      allowReplyOnClosed: response.settings.allowReplyOnClosed,
      createTicketRateLimitHits: response.settings.createTicketRateLimitHits,
      createTicketRateLimitWindowSeconds: response.settings.createTicketRateLimitWindowSeconds,
      maxOpenTicketsPerUser: response.settings.maxOpenTicketsPerUser,
    }));
  };

  const handleSaveSettings = async () => {
    try {
      setSettingsSaving(true);
      const settings = await updateAdminSettings(settingsForm);
      setBootstrap((current) => (current ? { ...current, settings: settings.settings } : current));
      setSettingsForm((current) => ({
        ...current,
        categoriesEnabled: settings.settings.categoriesEnabled,
        allowClientClose: settings.settings.allowClientClose,
        allowReplyOnClosed: settings.settings.allowReplyOnClosed,
        createTicketRateLimitHits: settings.settings.createTicketRateLimitHits,
        createTicketRateLimitWindowSeconds: settings.settings.createTicketRateLimitWindowSeconds,
        maxOpenTicketsPerUser: settings.settings.maxOpenTicketsPerUser,
        discordWebhookEnabled: settings.discordWebhook.enabled,
        discordWebhookUrl: settings.discordWebhook.webhookUrl ?? '',
        discordNotifyOnTicketCreated: settings.discordWebhook.notifyOnTicketCreated,
        discordNotifyOnClientReply: settings.discordWebhook.notifyOnClientReply,
        discordNotifyOnStaffReply: settings.discordWebhook.notifyOnStaffReply,
        discordNotifyOnInternalNote: settings.discordWebhook.notifyOnInternalNote,
        discordNotifyOnStatusChange: settings.discordWebhook.notifyOnStatusChange,
        discordNotifyOnAssignmentChange: settings.discordWebhook.notifyOnAssignmentChange,
        discordNotifyOnTicketDeleted: settings.discordWebhook.notifyOnTicketDeleted,
      }));
      addToast('Support settings saved.', 'success');
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setSettingsSaving(false);
    }
  };

  const handleSaveCategory = async () => {
    try {
      setCategorySaving(true);
      const category = await upsertAdminCategory({
        uuid: selectedCategoryUuid || undefined,
        name: categoryForm.name,
        description: categoryForm.description || undefined,
        color: categoryForm.color || undefined,
        sortOrder: categoryForm.sortOrder,
        enabled: categoryForm.enabled,
      });

      await refreshBootstrap();
      setSelectedCategoryUuid(category.uuid);
      addToast(selectedCategoryUuid ? 'Category updated.' : 'Category created.', 'success');
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setCategorySaving(false);
    }
  };

  const handleDeleteCategory = async () => {
    if (!selectedCategoryUuid) {
      return;
    }

    try {
      setCategorySaving(true);
      await deleteAdminCategory(selectedCategoryUuid);
      await refreshBootstrap();
      setSelectedCategoryUuid(null);
      setCategoryDeleteOpen(false);
      addToast('Category deleted.', 'success');
    } catch (error) {
      addToast(httpErrorToHuman(error), 'error');
    } finally {
      setCategorySaving(false);
    }
  };

  if (loading) {
    return <Spinner.Centered />;
  }

  if (fatalError || !bootstrap) {
    return (
      <ScreenBlock title='Support Settings Unavailable' content={fatalError ?? 'Unable to load support settings.'} />
    );
  }

  return (
    <>
      <ConfirmationModal
        opened={categoryDeleteOpen}
        onClose={() => setCategoryDeleteOpen(false)}
        title='Delete Category'
        confirm='Delete Category'
        onConfirmed={handleDeleteCategory}
      >
        <Text size='sm'>
          Deleting a category does not delete tickets. Tickets simply lose that category assignment.
        </Text>
      </ConfirmationModal>

      <Stack gap='md' mt='md'>
        <Group justify='space-between' align='end'>
          <div>
            <Title order={2} c='white'>
              Support Settings
            </Title>
            <Text c='dimmed'>
              Configure client ticket behavior, category structure, and the support workflow behind the ticket queue.
            </Text>
          </div>

          <Button variant='light' color='gray' onClick={() => navigate('/admin/support')}>
            Open Ticket Queue
          </Button>
        </Group>

        <Card p='md'>
          <Text fw={700}>Overview</Text>
          <Text c='dimmed' size='sm'>
            This page controls the support system extension configuration mounted at
            `/admin/extensions/dev.luxxy.supportsystem`.
          </Text>

          <SimpleGrid cols={{ base: 1, md: 2, xl: 4 }} mt='sm'>
            <div>
              <Text size='xs' c='dimmed'>
                Categories
              </Text>
              <Text fw={600}>{bootstrap.categories.length}</Text>
            </div>
            <div>
              <Text size='xs' c='dimmed'>
                Enabled Categories
              </Text>
              <Text fw={600}>{enabledCategoryCount}</Text>
            </div>
            <div>
              <Text size='xs' c='dimmed'>
                Staff Users
              </Text>
              <Text fw={600}>{bootstrap.staffUsers.length}</Text>
            </div>
            <div>
              <Text size='xs' c='dimmed'>
                Client Close
              </Text>
              <Text fw={600}>{bootstrap.settings.allowClientClose ? 'Enabled' : 'Disabled'}</Text>
            </div>
            <div>
              <Text size='xs' c='dimmed'>
                Max Open Tickets
              </Text>
              <Text fw={600}>
                {bootstrap.settings.maxOpenTicketsPerUser > 0 ? bootstrap.settings.maxOpenTicketsPerUser : 'Unlimited'}
              </Text>
            </div>
          </SimpleGrid>
        </Card>

        {!canManageSettings && !canManageCategories ? (
          <ScreenBlock
            title='No Settings Access'
            content='Your role does not currently have support settings or category management permissions.'
          />
        ) : (
          <SimpleGrid cols={{ base: 1, xl: 2 }}>
            {canManageSettings && (
              <Stack gap='md'>
                <Card p='md'>
                  <Text fw={700} mb='md'>
                    Ticket Settings
                  </Text>
                  <Stack gap='sm'>
                    <Switch
                      label='Enable categories for clients'
                      checked={settingsForm.categoriesEnabled}
                      onChange={(event) =>
                        setSettingsForm((current) => ({
                          ...current,
                          categoriesEnabled: event.currentTarget.checked,
                        }))
                      }
                    />
                    <Switch
                      label='Allow clients to close tickets'
                      checked={settingsForm.allowClientClose}
                      onChange={(event) =>
                        setSettingsForm((current) => ({
                          ...current,
                          allowClientClose: event.currentTarget.checked,
                        }))
                      }
                    />
                    <Switch
                      label='Allow replies on closed tickets'
                      checked={settingsForm.allowReplyOnClosed}
                      onChange={(event) =>
                        setSettingsForm((current) => ({
                          ...current,
                          allowReplyOnClosed: event.currentTarget.checked,
                        }))
                      }
                    />
                    <NumberInput
                      label='Create Ticket Rate Limit Hits'
                      description='How many new tickets a user can open per rate limit window. Set to 0 to disable this limiter.'
                      min={0}
                      max={10000}
                      value={settingsForm.createTicketRateLimitHits}
                      onChange={(value) =>
                        setSettingsForm((current) => ({
                          ...current,
                          createTicketRateLimitHits: typeof value === 'number' ? value : 0,
                        }))
                      }
                    />
                    <NumberInput
                      label='Create Ticket Rate Limit Window'
                      description='Window length in seconds for the ticket creation limiter.'
                      min={1}
                      max={86400}
                      value={settingsForm.createTicketRateLimitWindowSeconds}
                      onChange={(value) =>
                        setSettingsForm((current) => ({
                          ...current,
                          createTicketRateLimitWindowSeconds: typeof value === 'number' ? value : 300,
                        }))
                      }
                    />
                    <NumberInput
                      label='Max Open Tickets Per User'
                      description='Set to 0 to allow unlimited open tickets per user.'
                      min={0}
                      max={1000}
                      value={settingsForm.maxOpenTicketsPerUser}
                      onChange={(value) =>
                        setSettingsForm((current) => ({
                          ...current,
                          maxOpenTicketsPerUser: typeof value === 'number' ? value : 0,
                        }))
                      }
                    />
                  </Stack>
                </Card>

                <Card p='md'>
                  <Text fw={700}>Discord Webhook</Text>
                  <Text c='dimmed' size='sm' mb='md'>
                    Send support notifications into Discord as embeds with direct ticket and server links.
                  </Text>

                  <Stack gap='sm'>
                    <Switch
                      label='Enable Discord webhook notifications'
                      checked={settingsForm.discordWebhookEnabled}
                      onChange={(event) =>
                        setSettingsForm((current) => ({
                          ...current,
                          discordWebhookEnabled: event.currentTarget.checked,
                        }))
                      }
                    />

                    <TextInput
                      label='Discord Webhook URL'
                      placeholder='https://discord.com/api/webhooks/...'
                      value={settingsForm.discordWebhookUrl}
                      onChange={(event) =>
                        setSettingsForm((current) => ({
                          ...current,
                          discordWebhookUrl: event?.currentTarget?.value ?? '',
                        }))
                      }
                    />

                    <SimpleGrid cols={{ base: 1, md: 2 }}>
                      <Switch
                        label='Notify on new ticket'
                        checked={settingsForm.discordNotifyOnTicketCreated}
                        onChange={(event) =>
                          setSettingsForm((current) => ({
                            ...current,
                            discordNotifyOnTicketCreated: event.currentTarget.checked,
                          }))
                        }
                      />
                      <Switch
                        label='Notify on client reply'
                        checked={settingsForm.discordNotifyOnClientReply}
                        onChange={(event) =>
                          setSettingsForm((current) => ({
                            ...current,
                            discordNotifyOnClientReply: event.currentTarget.checked,
                          }))
                        }
                      />
                      <Switch
                        label='Notify on staff reply'
                        checked={settingsForm.discordNotifyOnStaffReply}
                        onChange={(event) =>
                          setSettingsForm((current) => ({
                            ...current,
                            discordNotifyOnStaffReply: event.currentTarget.checked,
                          }))
                        }
                      />
                      <Switch
                        label='Notify on internal note'
                        checked={settingsForm.discordNotifyOnInternalNote}
                        onChange={(event) =>
                          setSettingsForm((current) => ({
                            ...current,
                            discordNotifyOnInternalNote: event.currentTarget.checked,
                          }))
                        }
                      />
                      <Switch
                        label='Notify on status change'
                        checked={settingsForm.discordNotifyOnStatusChange}
                        onChange={(event) =>
                          setSettingsForm((current) => ({
                            ...current,
                            discordNotifyOnStatusChange: event.currentTarget.checked,
                          }))
                        }
                      />
                      <Switch
                        label='Notify on assignment change'
                        checked={settingsForm.discordNotifyOnAssignmentChange}
                        onChange={(event) =>
                          setSettingsForm((current) => ({
                            ...current,
                            discordNotifyOnAssignmentChange: event.currentTarget.checked,
                          }))
                        }
                      />
                    </SimpleGrid>

                    <Switch
                      label='Notify on ticket deletion'
                      checked={settingsForm.discordNotifyOnTicketDeleted}
                      onChange={(event) =>
                        setSettingsForm((current) => ({
                          ...current,
                          discordNotifyOnTicketDeleted: event.currentTarget.checked,
                        }))
                      }
                    />

                    <Group justify='flex-end'>
                      <Button color='blue' loading={settingsSaving} onClick={handleSaveSettings}>
                        Save Settings
                      </Button>
                    </Group>
                  </Stack>
                </Card>
              </Stack>
            )}

            {canManageCategories && (
              <Card p='md'>
                <Text fw={700} mb='md'>
                  Category Manager
                </Text>
                <Stack gap='sm'>
                  <MantineSelect
                    label='Edit Existing Category'
                    data={categoryOptions}
                    value={selectedCategoryUuid}
                    allowDeselect
                    clearable
                    searchable
                    onChange={setSelectedCategoryUuid}
                  />

                  <TextInput
                    label='Name'
                    value={categoryForm.name}
                    onChange={(event) =>
                      setCategoryForm((current) => ({ ...current, name: event?.currentTarget?.value ?? '' }))
                    }
                  />

                  <TextInput
                    label='Description'
                    value={categoryForm.description}
                    onChange={(event) =>
                      setCategoryForm((current) => ({ ...current, description: event?.currentTarget?.value ?? '' }))
                    }
                  />

                  <TextInput
                    label='Color'
                    placeholder='#4f46e5'
                    value={categoryForm.color}
                    onChange={(event) =>
                      setCategoryForm((current) => ({ ...current, color: event?.currentTarget?.value ?? '' }))
                    }
                  />

                  <NumberInput
                    label='Sort Order'
                    value={categoryForm.sortOrder}
                    onChange={(value) =>
                      setCategoryForm((current) => ({
                        ...current,
                        sortOrder: typeof value === 'number' ? value : 0,
                      }))
                    }
                  />

                  <Switch
                    label='Enabled'
                    checked={categoryForm.enabled}
                    onChange={(event) =>
                      setCategoryForm((current) => ({ ...current, enabled: event.currentTarget.checked }))
                    }
                  />

                  <Group justify='space-between'>
                    <Button
                      color='red'
                      variant='light'
                      disabled={!selectedCategoryUuid}
                      onClick={() => setCategoryDeleteOpen(true)}
                    >
                      Delete Category
                    </Button>

                    <Button color='blue' loading={categorySaving} onClick={handleSaveCategory}>
                      {selectedCategoryUuid ? 'Save Category' : 'Create Category'}
                    </Button>
                  </Group>
                </Stack>
              </Card>
            )}
          </SimpleGrid>
        )}
      </Stack>
    </>
  );
}
