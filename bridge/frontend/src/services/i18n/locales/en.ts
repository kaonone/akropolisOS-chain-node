// tslint:disable:max-line-length
const en = {
  app: {
    mainTitle: 'Ethereum DAI <--> AkropolisOS Bridge',
    pages: {
      limits: {
        title: 'Limits',
      },
      validators: {
        title: 'Validators',
      },
      settings: {
        title: 'Local settings',
      },
    },
  },
  utils: {
    validation: {
      isRequired: 'Field is required',
      isValidSubstrateAddress: 'Enter a valid Substrate address',
      isValidEthereumAddress: 'Enter a valid Ethereum address',
      isNumber: 'Enter a valid number',
      decimalsMoreThen: 'Enter a valid number with decimals less than %{decimals} digits',
      isValidNodeUrl: 'Node url should starts with "wss://"',
    },
  },
  components: {
    pagination: {
      itemsPerPage: 'Items per page',
      currentPagination: '%{from} - %{to} of %{total}',
    },
  },
  features: {
    transfersList: {
      title: 'Transfers',
      direction: 'Direction',
      ethAddress: 'Ethereum address',
      subAddress: 'Substrate address',
      amount: 'Amount',
      status: 'Status',
      blockNumber: 'Block number',
      notFound: 'Transfers not found',
    },
    tokenTransfer: {},
    settings: {
      localSettings: {
        bridgeSettings: 'Bridge settings',
        connectionStatus: 'Connection status:',
        resetButton: 'Reset & Reload',
        saveButton: 'Save & reload',
      },
      limits: {
        notFound: 'Limits not found',
        kind: 'Limit',
        value: 'Value',
        ethBlockNumber: 'Block number',
      },
    },
  },
};

export { en };
