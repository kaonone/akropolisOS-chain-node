// tslint:disable:max-line-length
const en = {
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
    tokenTransfer: {
      settings: {
        localSettigs: 'Local settings',
        bridgeSettings: 'Bridge settings',
        connectionStatus: 'Connection status:',
        resetButton: 'Reset & Reload',
        saveButton: 'Save & reload',
      },
    },
  },
};

export { en };
