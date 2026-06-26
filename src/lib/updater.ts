import { getVersion } from '@tauri-apps/api/app';

export interface UpdateInfo {
  currentVersion: string;
  availableVersion: string;
  notes?: string;
  pubDate?: string;
}

export async function checkForUpdate(
  opts: { timeout?: number } = {},
): Promise<
  { status: 'up-to-date' } | { status: 'available'; info: UpdateInfo }
> {
  const { check } = await import('@tauri-apps/plugin-updater');

  const currentVersion = await getVersion();
  const update = await check({ timeout: opts.timeout ?? 30000 });

  if (!update) {
    return { status: 'up-to-date' };
  }

  return {
    status: 'available',
    info: {
      currentVersion,
      availableVersion: update.version,
      notes: update.body ?? undefined,
      pubDate: update.date ?? undefined,
    },
  };
}
