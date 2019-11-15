import { tKeys, ITranslateKey } from 'services/i18n';

const floatRegExp = /^\d+?([.]|[.]\d+)?$/;

function makeFloatDecimalsRegExp(decimals: number) {
  return new RegExp(`^\\d+?([.]|[.]\\d{1,${decimals}})?$`);
}

export function validateFloat(value: string, decimals: number): ITranslateKey | undefined {
  return (
    (!floatRegExp.test(value) && tKeys.utils.validation.isNumber.getKey()) ||
    (!makeFloatDecimalsRegExp(decimals).test(value) && {
      key: tKeys.utils.validation.decimalsMoreThen.getKey(),
      params: { decimals },
    }) ||
    undefined
  );
}
