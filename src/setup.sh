# Update and upgrade
apt-get update
apt-get upgrade -y

# Install dependencies
apt-get install -y unzip tree

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash
apt-get install nodejs -y

# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
source /root/.bashrc
foundryup

# Install Go
wget https://go.dev/dl/go1.23.1.linux-amd64.tar.gz
rm -rf /usr/local/go && tar -C /usr/local -xzf go1.23.1.linux-amd64.tar.gz

# Install Medusa
go install github.com/crytic/medusa@latest

# Install Echidna
ECHIDNA_VERSION="2.2.6"
wget https://github.com/crytic/echidna/releases/download/v${ECHIDNA_VERSION}/echidna-${ECHIDNA_VERSION}-x86_64-linux.tar.gz
tar -xzvf echidna-${ECHIDNA_VERSION}-x86_64-linux.tar.gz
rm echidna-${ECHIDNA_VERSION}-x86_64-linux.tar.gz
mv echidna /usr/local/
mv /usr/local/echidna /usr/local/bin/



