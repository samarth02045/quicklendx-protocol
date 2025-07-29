# Pull Request Template

## 📝 Description
<!-- Provide a brief description of the changes made in this PR -->


## 🎯 Type of Change
<!-- Mark the type of change this PR represents -->
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] Performance improvement
- [ ] Security enhancement

## 🔧 Changes Made
<!-- List the specific changes made in this PR -->
- 
- 
- 

### Files Modified
<!-- List the main files that were modified -->
- 
- 
- 

### New Functionality Added
<!-- Describe any new features or functionality -->
- 
- 

## 🧪 Testing
<!-- Ensure all testing requirements are met -->
- [ ] Unit tests pass (`cargo test` in quicklendx-contracts/)
- [ ] Integration tests pass
- [ ] Manual testing completed
- [ ] No breaking changes introduced
- [ ] Test coverage maintained or improved

### Test Commands Run
```bash
# Add the commands you ran to test your changes
```

## 📋 Contract Checklist
<!-- For changes to Soroban smart contracts -->
- [ ] Soroban contract builds successfully (`stellar contract build`)
- [ ] WASM compilation works without errors
- [ ] Gas usage optimized and within reasonable limits
- [ ] Security considerations reviewed (no vulnerabilities introduced)
- [ ] Events properly emitted for state changes
- [ ] Error handling implemented with appropriate error codes
- [ ] Storage operations are efficient
- [ ] Contract functions have proper authorization checks

### Contract Build Output
```bash
# Paste the output of `stellar contract build` here if applicable
```

## 🔒 Security Checklist
<!-- Security considerations for the changes -->
- [ ] No sensitive data exposed in logs or events
- [ ] Input validation implemented where needed
- [ ] Authorization checks are in place
- [ ] No hardcoded secrets or private keys
- [ ] External dependencies are secure and up-to-date

## 📋 Review Checklist
<!-- General code quality and review requirements -->
- [ ] Code follows project style guidelines
- [ ] Code is self-documenting with clear variable/function names
- [ ] Documentation updated if needed (README, inline comments)
- [ ] No console.log or debug statements left in production code
- [ ] Error handling implemented appropriately
- [ ] Edge cases considered and handled
- [ ] Performance implications considered
- [ ] Backward compatibility maintained (unless breaking change)

## 📚 Documentation
<!-- Documentation updates -->
- [ ] README updated if needed
- [ ] API documentation updated
- [ ] Inline code comments added for complex logic
- [ ] CONTRIBUTING.md followed

## 🔗 Related Issues
<!-- Link to related issues -->
Closes #
Related to #

## 📸 Screenshots/Demo
<!-- Add screenshots or demo links if applicable -->


## 🚀 Deployment Notes
<!-- Any special deployment considerations -->
- [ ] No database migrations required
- [ ] No environment variable changes needed
- [ ] No breaking changes to existing APIs
- [ ] Safe to deploy to production

## 📝 Additional Notes
<!-- Any additional information for reviewers -->


## 🙋‍♂️ Questions for Reviewers
<!-- Specific questions or areas where you'd like focused review -->
- 
- 

---

### Reviewer Guidelines
- [ ] Code review completed
- [ ] Tests reviewed and verified
- [ ] Security implications considered
- [ ] Documentation is adequate
- [ ] Ready for merge

**Note**: Please ensure all checkboxes are completed before requesting review. This helps maintain code quality and speeds up the review process.