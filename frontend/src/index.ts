import { faLifeRing } from '@fortawesome/free-solid-svg-icons';
import type { FC } from 'react';
import { Extension, ExtensionContext } from 'shared';
import '@mantine/tiptap/styles.css';
import AdminSupportSettingsPage from './pages/AdminSupportSettingsPage.tsx';
import AdminSupportTicketPage from './pages/AdminSupportTicketPage.tsx';
import AdminSupportTicketsPage from './pages/AdminSupportTicketsPage.tsx';
import DashboardSupportCreatePage from './pages/DashboardSupportCreatePage.tsx';
import DashboardSupportTicketPage from './pages/DashboardSupportTicketPage.tsx';
import DashboardSupportTicketsPage from './pages/DashboardSupportTicketsPage.tsx';

class DevLuxxySupportSystemExtension extends Extension {
  public cardConfigurationPage: FC | null = AdminSupportSettingsPage;
  public cardComponent: FC | null = null;

  public initialize(ctx: ExtensionContext): void {
    ctx.extensionRegistry.routes.addAccountRoute({
      name: 'Support',
      icon: faLifeRing,
      path: '/support',
      exact: true,
      element: DashboardSupportTicketsPage,
    });

    ctx.extensionRegistry.routes.addAccountRoute({
      name: undefined,
      path: '/support/new',
      element: DashboardSupportCreatePage,
    });

    ctx.extensionRegistry.routes.addAccountRoute({
      name: undefined,
      path: '/support/:ticket',
      element: DashboardSupportTicketPage,
    });

    ctx.extensionRegistry.routes.addAdminRoute({
      name: 'Support',
      icon: faLifeRing,
      path: '/support',
      exact: true,
      permission: 'tickets.view-all',
      element: AdminSupportTicketsPage,
    });

    ctx.extensionRegistry.routes.addAdminRoute({
      name: undefined,
      path: '/support/:ticket',
      permission: 'tickets.view-all',
      element: AdminSupportTicketPage,
    });
  }
}

export default new DevLuxxySupportSystemExtension();
