import { useState, useEffect, useCallback, useRef } from 'react';
import { checkForUpdate } from '@/lib/settings-api';
import type { UpdateInfo } from '@/lib/settings-api';

let startupCheckStarted = false;

export function useUpdate() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [checking, setChecking] = useState(false);
  const [checked, setChecked] = useState(false);
  const mountedRef = useRef(true);

  const checkUpdate = useCallback(async () => {
    setChecking(true);
    try {
      const info = await checkForUpdate();
      if (mountedRef.current) {
        setUpdateInfo(info);
        setChecked(true);
      }
      return info;
    } catch (error) {
      console.error('Failed to check for update:', error);
      return null;
    } finally {
      if (mountedRef.current) setChecking(false);
    }
  }, []);

  useEffect(() => {
    mountedRef.current = true;
    if (!startupCheckStarted) {
      startupCheckStarted = true;
      checkUpdate();
    }
    return () => {
      mountedRef.current = false;
    };
  }, [checkUpdate]);

  return {
    updateInfo,
    hasUpdate: !!updateInfo,
    checking,
    checked,
    checkUpdate,
  };
}
