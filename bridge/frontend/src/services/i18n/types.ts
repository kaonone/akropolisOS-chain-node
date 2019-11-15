import Polyglot from 'node-polyglot';

import { tKeys } from './constants';

type CustomTranslateFunction = (phrase: ITranslateKey) => string;
export interface IPhraseWithOptions {
  key: string;
  params: Record<string, string | number>;
}

export type ITranslateFunction = Polyglot['t'] & CustomTranslateFunction;
export type ITranslateKey = string | IPhraseWithOptions;

export type Lang = 'en' | 'ru';

export interface ITranslateProps {
  locale: Lang;
  tKeys: typeof tKeys;
  t: ITranslateFunction;
  changeLanguage: null | ((locale: Lang) => void);
}
