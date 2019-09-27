import { RegistryTypes } from '@polkadot/types/types';

interface INetworkConfig {
  id: number;
  name: string;
  rpcUrl: string;
  contracts: {
    bridge: string;
    dai: string;
  };
}

const ethNetworkConfigs: Record<number, INetworkConfig> = {
  "42": {
    id: 42,
    name: "Kovan",
    rpcUrl: "https://kovan.infura.io/",
    contracts: {
      bridge: "0x9ff8c644F09B0B7dc030C8aaD52dC1628a22C4c2",
      dai: "0xC4375B7De8af5a38a93548eb8453a498222C4fF2"
    }
  },
  "1": {
    id: 1,
    name: "Mainnet",
    rpcUrl: "https://mainnet.infura.io/",
    contracts: {
      bridge: "0x9ff8c644F09B0B7dc030C8aaD52dC1628a22C4c2",
      dai: "0x89d24a6b4ccb1b6faa2625fe562bdd9a23260359"
    }
  }
};

export const NETWORK_ID = 42;
export const ETH_NETWORK_CONFIG = ethNetworkConfigs[NETWORK_ID];
export const DEFAULT_DECIMALS = 18;

export const SUBSTRATE_NODE_URL = 'wss://node1-chain.akropolis.io';
export const SUBSTRATE_NODE_CUSTOM_TYPES: RegistryTypes = {
  Count: 'u64',
  DaoId: 'u64',
  MemberId: 'u64',
  ProposalId: 'u64',
  VotesCount: 'MemberId',
  Days: 'u32',
  Rate: 'u32',
  Dao: {
    address: 'AccountId',
    name: 'Text',
    description: 'Bytes',
    founder: 'AccountId',
  },
  Action: {
    _enum: {
      EmptyAction: null,
      AddMember: 'AccountId',
      RemoveMember: 'AccountId',
      GetLoan: '(Vec<u8>, Days, Rate, Balance)',
      Withdraw: '(AccountId, Balance, Vec<u8>)',
    },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  } as any, // because RegistryTypes is wrong
  Proposal: {
    dao_id: 'DaoId',
    action: 'Action',
    open: 'bool',
    accepted: 'bool',
    voting_deadline: 'BlockNumber',
    yes_count: 'VotesCount',
    no_count: 'VotesCount',
  },
};
