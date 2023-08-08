# The Auditor

NOTE: This is not an official Veeam tool. It is provided under MIT, use at your own risk.

The Auditor is a tool for setting up and managing Veeam Backup for M365 (VB365) audit notifications.

These notifications have to be set up using the VB365 Rest API which can be found here:

https://helpcenter.veeam.com/docs/vbo365/rest/reference/vbo365-rest.html?ver=70#tag/OrganizationAudit

Credit also goes to Jorge De La Cruz for his blog post on this subject which can be found here:

https://jorgedelacruz.uk/2023/08/04/veeam-veeam-backup-for-microsoft-365-security-notifications-for-restore-operations-modern-auth/

The Auditor is only set up to work with Microsoft notifications via the Graph API, it does not support the legacy Azure smtp notifications or Google notifications.

NOTE: The Auditor only works with VB365 v7.

## Installation

The Auditor is a Rust program which you can install by downloading this repo and running:

```
cargo install --path .
```

You will need Rust installed to do this, which you can get from here:

https://www.rust-lang.org/tools/install

You can then run the Auditor by running:

```
the_auditor
```

From any location as it will be added to your path.

To uninstall run.

```
cargo uninstall the_auditor
```

## Azure AD App

You will need to create an Azure AD application to use the Auditor. You can follow the steps in Jorge's blog post to do this:

https://jorgedelacruz.uk/2023/08/04/veeam-veeam-backup-for-microsoft-365-security-notifications-for-restore-operations-modern-auth/

## Configuration

You will need to set a environment variable called "VEEAM_API_PASSWORD" which is the password to your Veeam Backup for M365 account.

You will also need a config.toml file in the directory that you run the Auditor from. This file should look like this:

```
[azure]
tenant_id = "" # optional
client_id = "" # optional
client_secret = "" # optional
redirect_url = "http://localhost" # use if you do not assign the above

[notification]
username = ""
user_id = ""
from = ""
to = ""
subject = ""

[vb365]
username = ""
address = ""
```

| Object       | Key           | Description                                                                                                                            |
| ------------ | ------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| azure        | tenant_id     | The tenant id of your Azure AD - Optional                                                                                              |
| azure        | client_id     | The client id of your Azure AD app - Optional                                                                                          |
| azure        | client_secret | The client secret of your Azure AD app - Optional                                                                                      |
| azure        | redirect_url  | The redirect url of your Azure AD app - Use http://localhost if you do not assign the above.                                           |
| notification | user_id       | Specifies an authenticated user account ID. Veeam Backup for Microsoft 365 will send audit email notifications on behalf of this user. |
| notification | from          | Specifies email address of the notification sender.                                                                                    |
| notification | to            | Specifies email address of the notification recipient. For listing multiple recipients, use semicolon as a separator.                  |
| notification | subject       | Specifies the subject for audit email notifications.                                                                                   |
| vb365        | username      | The username of your Veeam M365 account                                                                                                |
| vb365        | address       | The address of your Veeam M365 server                                                                                                  |

NOTE: In most cases you will not need to set the tenant_id, client_id or client_secret as VB365 will use the assigned Azure AD app.

See:

https://helpcenter.veeam.com/docs/vbo365/rest/reference/vbo365-rest.html?ver=70#tag/AuditEmailSettings/operation/AuditEmailSettings_Update

For more information on the notification settings.

## Options

If it can it will present you with a list of options:

| Option              | Description                                                            |
| ------------------- | ---------------------------------------------------------------------- |
| Get Audit Items     | Gets a list of all the audit items that have been set up in Veeam M365 |
| Add Audit Item      | Adds a new audit item to Veeam M365                                    |
| Remove Audit Item   | Removes an audit item from Veeam M365                                  |
| Get Users/Groups    | Gets a list of all the users and groups in your VB365 instance         |
| Setup Notifications | Sets up the Azure app and the notification settings in VB365           |
| Send Test Email     | Sends a test email to the notification recipient                       |

## Usage

### Setup

If it is your first use you will need to run the Setup option which will set up the notification authentication and the notification settings.

Doing this will trigger a web browser to open and you will need to log in to your Azure AD account remember to enabled "Consent on behalf of your organization".

This will then start a TCP listener on the port that you specified in the call back address. Once you have finished authenticating, the listener will automatically get the data from the callback
and use it in to complete the setup.

If it all works you will see a message saying "Notification settings updated successfully!".

### Getting Users and Groups

Before adding audit items you will need to get a list of users and groups from your VB365 instance. You can do this by running the Get Users/Groups command.

These will save either a users.json or groups.json file in the directory that you run the Auditor from.

You will need to remove the users and groups that you don't want to audit from these files. These items are in the "results" array in both cases.

### Adding Audit Items

Once you have the users.json and groups.json files set up you can run the Add Audit Item command.

It will ask if you want to add Users or Groups, it will then read either the users.json or groups.json file and add them to the audit items.

### Removing Audit Items

If you want to remove an audit item you can run the Remove Audit Item command.

It will first ask if you want to remove a user or a group, it will then present you with a multiselect list of the audit items, you can then select the ones you want to remove.

### List Audit Items

To check the items that are being audited you can run the Get Audit Items command.

### Testing Notifications

You can then run the Send Test Email command to test that the notifications are working.

## Authentication with VB365

The Auditor uses the VB365 Rest API.

https://helpcenter.veeam.com/docs/vbo365/rest/reference/vbo365-rest.html?ver=70

The tool will save a file called "token.json" in the directory that you run TShe Auditor from. This file contains the access token that is used to authenticate with VB365 as well as when it will expire.

The process the tool uses to authenticate is as follows:

- Check if the token.json file exists
  - If it does check if the token is still valid
    - If it is, use it
    - If it isn't get a new token and save it to the token.json file
  - If it doesn't get a new token and save it to the token.json file

Doing this saves a lot of new tokens being generated.

## Issues/Contributions

If you have any issues or would like to contribute please raise an issue or a pull request.

## Feature Requests

If you have any feature requests please raise an issue.
