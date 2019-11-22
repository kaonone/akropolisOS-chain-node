import { tKeys, ITranslateKey } from 'services/i18n';

export function validateNodeUrl(value: string): ITranslateKey | undefined {
  return value && /^wss:\/\/\w+$/.test(value)
    ? undefined
    : tKeys.utils.validation.isValidNodeUrl.getKey();
}
