import { tKeys, ITranslateKey } from 'services/i18n';

export function validateRequired(value: any): ITranslateKey | undefined {
  return value ? undefined : tKeys.utils.validation.isRequired.getKey();
}
