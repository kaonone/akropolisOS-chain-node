pragma solidity ^0.5.9;

import 'openzeppelin-solidity/contracts/token/ERC20/SafeERC20.sol';

//Beneficieries (validators) template
import "../helpers/BeneficiaryOperations.sol";

contract DAIBridge is  BeneficiaryOperations {

        IERC20 private token;

    enum Status {PENDING,WITHDRAW,APPROVED, CANCELED, CONFIRMED}

        struct Message {
            bytes32 messageID;
            address spender;
            bytes32 substrateAddress;
            uint availableAmount;
            Status status;
        }

        event RelayMessage(bytes32 messageID, address sender, bytes32 recipient, uint amount);
        event WithdrawFromBridge(bytes32 MessageID, address sender, uint amount);
        event ApprovedRelayMessage(bytes32 messageID, address  sender,  bytes32 recipient, uint amount);


        mapping(bytes32 => Message) messages;
        mapping(address => Message) messagesBySender;

       /**
       * @notice Constructor.
       * @param _token  Address of DAI token
       */

        constructor (IERC20 _token) public
            BeneficiaryOperations() {
            token = _token;
        }  

        // MODIFIERS
        /**
        * @dev Allows to perform method by existing beneficiary
        */

        modifier onlyExistingBeneficiary(address _beneficiary) {
            require(isExistBeneficiary(_beneficiary), "address is not in beneficiary array");
             _;
        }

        /*
            check available amount
        */

        modifier messageHasAmount(bytes32 messageID) {
            require((messages[messageID].availableAmount > 0), "Amount withdraw");
            _;
        }

        /*
            check that message is valid
        */
        modifier validMessage(bytes32 messageID, address spender, bytes32 substrateAddress, uint availableAmount) {
            require((messages[messageID].spender == spender)
                && (messages[messageID].substrateAddress == substrateAddress)
                && (messages[messageID].availableAmount == availableAmount), "data is not valid");
            _;
        }

        modifier pendingMessage(bytes32 messageID) {
            require(messages[messageID].status ==  Status.PENDING, "Message is not pending");
            _;
        }

         modifier approvedMessage(bytes32 messageID) {
            require(messages[messageID].status ==  Status.APPROVED, "Message is not approved");
            _;
        }

        /*
        * Set Transfer to Bridge
        */

        function setTransfer(uint amount, bytes32 substrateAddress) public {
            require(token.allowance(msg.sender, address(this)) >= amount, "contract is not allowed to this amount");
            token.transferFrom(msg.sender, address(this), amount);

            bytes32 messageID = keccak256(abi.encodePacked(now));

            Message  memory message = Message(messageID, msg.sender, substrateAddress, amount, Status.PENDING);

            emit RelayMessage(messageID, msg.sender, substrateAddress, amount);
        }

        function withdraw(bytes32 messageID) public pendingMessage(messageID) {
            Message storage message = messages[messageID];

            message.status = Status.WITHDRAW;

            token.transfer(msg.sender, message.availableAmount);

            emit WithdrawFromBridge(messageID, msg.sender, message.availableAmount);
        }

        function approveTransfer(bytes32 messageID, address spender, bytes32 substrateAddress, uint availableAmount)
            public validMessage(messageID, spender, substrateAddress, availableAmount) pendingMessage(messageID )onlyManyBeneficiaries {
            Message storage message = messages[messageID];

            emit ApprovedRelayMessage(messageID, spender, substrateAddress, availableAmount);

            message.status = Status.APPROVED;
        }

}