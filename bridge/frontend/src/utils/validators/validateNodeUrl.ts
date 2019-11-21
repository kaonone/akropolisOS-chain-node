import { tKeys, ITranslateKey } from 'services/i18n';

export function validateNodeUrl(value: string): ITranslateKey | undefined {
  return value && value.substr(0, 6) === 'wss://'
    ? undefined
    : tKeys.utils.validation.isValidNodeUrl.getKey();
}
