pragma solidity ^0.5.9;

import 'openzeppelin-solidity/contracts/token/ERC20/SafeERC20.sol';

//Beneficieries (validators) template
import "../helpers/BeneficiaryOperations.sol";

contract DAIBridge is  BeneficiaryOperations {

        IERC20 private token;

        struct Message {
            bytes32 messageID;
            address spender;
            string substrateAddress;
            uint availableAmount;
        }

        event RelayMessage(bytes32 messageID, address indexed sender, string indexed recipient, uint amount);
        event WithdrawFromBridge(address indexed sender, uint amount);

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
        modifier validMessage(bytes32 messageID, address spender, string memory substrateAddress, uint availableAmount) {
            require((messages[messageID].spender == spender)
                && (keccak256(abi.encodePacked(messages[messageID].substrateAddress)) == keccak256(abi.encodePacked(substrateAddress)))
                && (messages[messageID].availableAmount == availableAmount), "data is not valid");
            _;
        }



        /*
        * Set Transfer to Bridge
        */

        function setTransfer(uint amount, string memory substrateAddress) public {
            require(token.allowance(msg.sender, address(this)) >= amount, "contract is not allowed to this amount");
            token.transferFrom(msg.sender, address(this), amount);

            bytes32 messageID = keccak256(abi.encodePacked(now));

            Message  memory message = Message(messageID, msg.sender, substrateAddress, amount);

            emit RelayMessage(messageID, msg.sender, substrateAddress, amount);
        }

        function withdraw() public {
            Message storage message = messagesBySender[msg.sender];

            token.transfer(msg.sender, message.availableAmount);

            message.availableAmount = 0;

            emit WithdrawFromBridge(msg.sender, message.availableAmount);
        }




}