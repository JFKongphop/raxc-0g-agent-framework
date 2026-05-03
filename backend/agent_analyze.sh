#!/bin/bash
# Test RAXC Step 9.9 API with VulnerableVault contract

API_URL="http://localhost:3000"

echo "╔══════════════════════════════════════════════════════════════════════════╗"
echo "║          RAXC Step 9.9 API - VulnerableVault Analysis Test               ║"
echo "╚══════════════════════════════════════════════════════════════════════════╝"
echo ""

# Send POST request with VulnerableVault contract
echo "[*] Sending contract to API for analysis..."
echo ""

RESPONSE=$(curl -s -X POST "$API_URL/analyze" \
  -H "Content-Type: application/json" \
  -d '{
    "contract": "pragma solidity ^0.7.0;\n\ncontract VulnerableVault {\n    mapping(address => uint256) public balances;\n\n    function deposit() external payable {\n        balances[msg.sender] += msg.value;\n    }\n\n    function withdraw() external {\n        uint256 amount = balances[msg.sender];\n        require(amount > 0, \"Nothing to withdraw\");\n        // VULNERABILITY: external call before state update — reentrancy risk\n        (bool ok, ) = msg.sender.call{value: amount}(\"\");\n        require(ok, \"Transfer failed\");\n        balances[msg.sender] = 0;  // state updated AFTER the call\n    }\n\n    function getPrice() external view returns (uint256) {\n        // single-block spot price — manipulable via flash loan\n        return address(this).balance;\n    }\n}",
    "name": "VulnerableVault"
  }')

# Check if response is valid JSON
if echo "$RESPONSE" | jq . > /dev/null 2>&1; then
  echo "[✓] Analysis complete!"
  echo ""
  echo "╔══════════════════════════════════════════════════════════════════════════╗"
  echo "║                        ANALYSIS RESULT                                   ║"
  echo "╚══════════════════════════════════════════════════════════════════════════╝"
  echo ""
  echo "$RESPONSE" | jq .
  
  # Extract download URL
  DOWNLOAD_URL=$(echo "$RESPONSE" | jq -r '.download_url')
  
  if [ "$DOWNLOAD_URL" != "null" ] && [ -n "$DOWNLOAD_URL" ]; then
    echo ""
    echo "╔══════════════════════════════════════════════════════════════════════════╗"
    echo "║                      DOWNLOADING REPORT                                  ║"
    echo "╚══════════════════════════════════════════════════════════════════════════╝"
    echo ""
    
    # Create reports directory if it doesn't exist
    mkdir -p reports
    
    # Extract filename from URL
    FILENAME=$(basename "$DOWNLOAD_URL")
    REPORT_FILE="reports/$FILENAME"
    
    echo "[*] Report available at: $DOWNLOAD_URL"
    echo "[*] Downloading report to $REPORT_FILE..."
    
    curl -s "$API_URL$DOWNLOAD_URL" -o "$REPORT_FILE"
    
    echo "[✓] Report saved to $REPORT_FILE"
    echo ""
    echo "╔══════════════════════════════════════════════════════════════════════════╗"
    echo "║                     REPORT PREVIEW (First 60 lines)                      ║"
    echo "╚══════════════════════════════════════════════════════════════════════════╝"
    echo ""
    head -n 60 "$REPORT_FILE"
    echo ""
    echo "..."
    echo ""
    echo "View full report:"
    echo "  cat $REPORT_FILE"
    echo "  open $REPORT_FILE    # macOS"
  else
    echo ""
    echo "[!] No download URL in response"
    exit 1
  fi
else
  echo "[!] Error: API returned invalid response"
  echo "$RESPONSE"
  exit 1
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════════════════╗"
echo "║                          TEST COMPLETE                                   ║"
echo "╚══════════════════════════════════════════════════════════════════════════╝"
