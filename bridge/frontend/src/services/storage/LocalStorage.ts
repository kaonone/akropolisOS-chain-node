import { PayloadByKey, StorageKey } from './types';

class LocalStorage {
  public static checkAvailability() {
    const testKey = '__test__';

    try {
      localStorage.setItem(testKey, '__test-value__');
      localStorage.removeItem(testKey);
      return true;
    } catch (e) {
      console.warn('Local storage is not available! Some features will be disabled!');
      return false;
    }
  }

  private isLocalStorageAvailable: boolean | null = null;

  constructor(version: string) {
    this.isLocalStorageAvailable = LocalStorage.checkAvailability();
    this.checkVersion(version);
  }

  public set<T extends StorageKey>(key: T, value: PayloadByKey[T]): void {
    if (!this.isLocalStorageAvailable) {
      return;
    }

    localStorage.setItem(key, JSON.stringify(value));
  }

  public get<T extends StorageKey>(key: T): PayloadByKey[T] | null;
  public get<T extends StorageKey>(key: T, fallback: PayloadByKey[T]): PayloadByKey[T];
  public get<T extends StorageKey>(key: T, fallback?: PayloadByKey[T]): PayloadByKey[T] | null {
    const _fallback = fallback || null;

    if (!this.isLocalStorageAvailable) {
      return _fallback;
    }

    const data = localStorage.getItem(key);

    try {
      return data ? JSON.parse(data) : _fallback;
    } catch (e) {
      console.error(
        `Error while parsing data from localstorage for key: ${key}.
        Error is: ${e.message}, stack is: ${e.stack}`,
      );
      return _fallback;
    }
  }

  public reset() {
    if (this.isLocalStorageAvailable) {
      localStorage.clear();
    }
  }

  private checkVersion(version: string) {
    const currentVersion = this.getVersion();
    if (currentVersion !== version) {
      this.reset();
      this.saveVersion(version);
    }
  }

  private getVersion() {
    return this.get('version');
  }

  private saveVersion(version: string) {
    this.set('version', version);
  }
}

export { LocalStorage };
