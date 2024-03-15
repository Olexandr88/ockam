#!/bin/bash
set -ex

# Change into ec2-user's home directory and use sudo to run the commands as ec2-user
cd /home/ec2-user
sudo -u ec2-user bash << 'EOS'
set -ex

# Install Ockam Command
curl --proto '=https' --tlsv1.2 -sSfL https://install.command.ockam.io | bash
source "$HOME/.ockam/env"

# Run `ockam project enroll ...`
#
# The `project enroll` command creates a new vault and generates a cryptographic identity with
# private keys stored in that vault.
#
# The enrollment ticket includes routes and identitifiers for the project membership authority
# and the project’s node that offers the relay service.
#
# The enrollment ticket also includes an enrollment token. The project enroll command
# creates a secure channel with the project membership authority and presents this enrollment token.
# The authority enrolls presented identity and returns a project membership credential.
#
# The command, stores this credential for later use and exits.
ockam project enroll "$ENROLLMENT_TICKET"

# Create an ockam node.
#
# Create an access control policy that only allows project members that possesses a credential with
# attribute iperf-outlet="true" to connect to TCP Portal Inlets on this node.
#
# Create a TCP Portal Inlet to iperf
# This makes the remote iperf available on localhost at - 127.0.0.1:15201
OCKAM_LOG_LEVEL=info ockam node create
ockam policy create --resource-type tcp-inlet --expression '(= subject.iperf-outlet "true")'
ockam tcp-inlet create --from 127.0.0.1:15201 --to iperf

sudo yum update -y
sudo yum install iperf3 -y

EOS
