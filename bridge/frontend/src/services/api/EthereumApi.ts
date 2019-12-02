import Web3 from 'web3';
import Contract from 'web3/eth/contract';
import { Observable, interval, from } from 'rxjs';
import { switchMap, skipWhile } from 'rxjs/operators';
import BN from 'bn.js';
import { decodeAddress } from '@polkadot/util-crypto';
import { u8aToHex } from '@polkadot/util';
import { autobind } from 'core-decorators';

import { ETH_NETWORK_CONFIG } from 'env';
import bridgeAbi from 'abis/bridge.json';
import erc20Abi from 'abis/erc20.json';
import { getContractData$ } from 'utils/ethereum';
import { Direction, Status } from 'generated/bridge-graphql';

import { TransfersApi } from './TransfersApi';
import { ICreateProposalOptions } from './types';

export class EthereumApi {
  private daiContract: Contract;
  private bridgeContract: Contract;

  constructor(private web3: Web3, private transfersApi: TransfersApi) {
    this.daiContract = new this.web3.eth.Contract(erc20Abi, ETH_NETWORK_CONFIG.contracts.dai);
    this.bridgeContract = new this.web3.eth.Contract(
      bridgeAbi,
      ETH_NETWORK_CONFIG.contracts.bridge,
    );
  }

  @autobind
  public async sendToSubstrate(fromAddress: string, to: string, amount: string): Promise<void> {
    await this.approveBridge(fromAddress, amount);
    await this.sendToBridge(fromAddress, to, amount);
  }

  @autobind
  // eslint-disable-next-line class-methods-use-this
  public getEthValidators$(): Observable<string[]> {
    return from([
      [
        '6a8357ae0173737209af59152ee30a786dbade70',
        '93880d6508e3ffee5a4376939d3322f2f11b56d1',
        '9194ad793e72052992f9a1b3b8eaef5463300f87',
      ],
    ]);

    /* return getContractData$<string[], string[]>(this._bridgeContract, 'validators', {
      eventsForReload: [['ValidatorShipTransferred']],
    }); */
  }

  @autobind
  public getTokenBalance$(address: string): Observable<BN> {
    const formattedAddress = address.toLowerCase();

    return getContractData$<string, BN>(this.daiContract, 'balanceOf', {
      args: [formattedAddress],
      eventsForReload: [
        ['Transfer', { filter: { _from: formattedAddress } }],
        ['Transfer', { filter: { _to: formattedAddress } }],
      ],
      convert: value => new BN(value),
    });
  }

  @autobind
  public getAccount$(): Observable<string | null> {
    return from(getEthAccount(this.web3)).pipe(
      skipWhile(account => !account),
      switchMap(() => interval(1000).pipe(switchMap(() => getEthAccount(this.web3)))),
    );
  }

  @autobind
  public async approveNewLimit(proposalId: string, fromAddress: string): Promise<void> {
    await this.daiContract.methods.approvedNewProposal(proposalId).send({ from: fromAddress }); // TODO need to test
  }

  @autobind
  public async createLimitProposal(options: ICreateProposalOptions): Promise<void> {
    const {
      fromAddress,
      minHostTransactionValue,
      maxHostTransactionValue,
      dayHostMaxLimit,
      dayHostMaxLimitForOneAddress,
      maxHostPendingTransactionLimit,
      minGuestTransactionValue,
      maxGuestTransactionValue,
      dayGuestMaxLimit,
      dayGuestMaxLimitForOneAddress,
      maxGuestPendingTransactionLimit,
    } = options;

    await this.daiContract.methods
      .createProposal(
        minHostTransactionValue,
        maxHostTransactionValue,
        dayHostMaxLimit,
        dayHostMaxLimitForOneAddress,
        maxHostPendingTransactionLimit,
        minGuestTransactionValue,
        maxGuestTransactionValue,
        dayGuestMaxLimit,
        dayGuestMaxLimitForOneAddress,
        maxGuestPendingTransactionLimit,
      )
      .send({ from: fromAddress }); // TODO need to test
  }

  private async approveBridge(fromAddress: string, amount: string): Promise<void> {
    const allowance: string = await this.daiContract.methods
      .allowance(fromAddress, ETH_NETWORK_CONFIG.contracts.bridge)
      .call();

    if (new BN(amount).lte(new BN(allowance))) {
      return;
    }

    await this.daiContract.methods
      .approve(ETH_NETWORK_CONFIG.contracts.bridge, amount)
      .send({ from: fromAddress });
  }

  private async sendToBridge(fromAddress: string, to: string, amount: string): Promise<void> {
    const formatedToAddress = u8aToHex(decodeAddress(to));
    const bytesAddress = this.web3.utils.hexToBytes(formatedToAddress);

    const result = await this.bridgeContract.methods
      .setTransfer(amount, bytesAddress)
      .send({ from: fromAddress });

    const id = result?.events?.RelayMessage?.returnValues?.messageID;

    id &&
      this.transfersApi.pushToSubmittedTransfers$({
        id,
        amount,
        direction: Direction.Eth2Sub,
        ethAddress: fromAddress,
        subAddress: to,
        status: Status.Pending,
      });
  }
}

async function getEthAccount(web3: Web3): Promise<string | null> {
  // Modern dapp browsers...
  if (window.ethereum) {
    try {
      // Request account access
      await window.ethereum.enable();
    } catch (error) {
      console.error('User denied account access');
      throw error;
    }
  }

  const accounts = await web3.eth.getAccounts();
  return accounts[0] || null;
}
