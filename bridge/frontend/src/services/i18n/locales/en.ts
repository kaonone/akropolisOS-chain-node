// tslint:disable:max-line-length
const en = {
  app: {
    mainTitle: 'Ethereum DAI <--> AkropolisOS Bridge',
    pages: {
      limits: {
        title: 'Limits',
        proposalsTitle: 'Limits proposals',
        limitsChangingFormTitle: 'Change limits',
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
      mustBeAnInteger: 'Enter an integer',
      isPositiveNumber: 'Must be positive number',
    },
  },
  components: {
    pagination: {
      itemsPerPage: 'Items per page',
      currentPagination: '%{from} - %{to} of %{total}',
    },
    votingCard: {
      blockNumber: 'Block number',
      from: 'From',
      needed: 'Needed votes',
      limits: 'Limits',
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
        items: {
          MIN_HOST_TRANSACTION_VALUE: 'Min host transaction value',
          MAX_HOST_TRANSACTION_VALUE: 'Max host transaction value',
          DAY_HOST_MAX_LIMIT: 'Day host max limit',
          DAY_HOST_MAX_LIMIT_FOR_ONE_ADDRESS: 'Day host max limit for one address',
          MAX_HOST_PENDING_TRANSACTION_LIMIT: 'Max host pending transaction limit',
          MIN_GUEST_TRANSACTION_VALUE: 'Min guest transaction value',
          MAX_GUEST_TRANSACTION_VALUE: 'Max guest transaction value',
          DAY_GUEST_MAX_LIMIT: 'Day guest max limit',
          DAY_GUEST_MAX_LIMIT_FOR_ONE_ADDRESS: 'Day guest max limit for one address',
          MAX_GUEST_PENDING_TRANSACTION_LIMIT: 'Max guest pending transaction limit',
        },
        notFound: 'Limits not found',
        kind: 'Limit',
        value: 'Value',
        ethBlockNumber: 'Block number',
      },
      limitsChangingForm: {
        cancelButtonText: 'Cancel',
      },
      limitsProposalsList: {
        notFound: 'Limits proposals not found',
        status: {
          APPROVED: 'approved',
          DECLINED: 'declined',
        },
        showLimits: 'Show proposed limits',
      },
    },
  },
};

export { en };
