//! `arn:partition:service:region:account-id:resource` formatted ARN

use std::iter::Iterator;
use std::{error, fmt};

/// `arn:partition:service:region:account-id:resource` formatted ARN
///
/// # Example
///
/// ~~~~
/// use arn::naive::NaiveArn;
///
/// let arn = NaiveArn::parse("arn:aws:ec2:us-east-1:123456789012:vpc/vpc-fd580e98").unwrap();
/// ~~~~
#[derive(Debug, PartialEq)]
pub struct NaiveArn<'a> {
    /// The partition that the resource is in. For standard AWS regions, the partition is "aws". If you have resources in
    /// other partitions, the partition is "aws-partitionname". For example, the partition for resources in the China
    /// (Beijing) region is "aws-cn".
    pub partition: &'a str,

    /// The service namespace that identifies the AWS product (for example, Amazon S3, IAM, or Amazon RDS). For a list of
    /// namespaces, see
    /// <http://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html#genref-aws-service-namespaces>.
    pub service: &'a str,

    /// The region the resource resides in. Note that the ARNs for some resources do not require a region, so this
    /// component might be omitted.
    pub region: Option<&'a str>,

    /// The ID of the AWS account that owns the resource, without the hyphens. For example, 123456789012. Note that the
    /// ARNs for some resources don't require an account number, so this component might be omitted.
    pub account_id: Option<&'a str>,

    /// The content of this part of the ARN varies by service. It often includes an indicator of the type of resource —
    /// for example, an IAM user or Amazon RDS database - followed by a slash (/) or a colon (:), followed by the
    /// resource name itself. Some services allows paths for resource names, as described in
    /// <http://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html#arns-paths>.
    pub resource: &'a str,
}

impl<'a> NaiveArn<'a> {
    pub fn parse(s: &'a str) -> Result<Self, ParseNaiveArnError> {
        let mut elements = s.splitn(6, ':');

        if elements.next() != Some("arn") {
            return Err(ParseNaiveArnError::MissingPrefix);
        }

        let partition = match elements.next() {
            None => return Err(ParseNaiveArnError::NotEnoughElements),
            Some("") => return Err(ParseNaiveArnError::MissingPartition),
            Some(partition) => partition.into(),
        };

        let service = match elements.next() {
            None => return Err(ParseNaiveArnError::NotEnoughElements),
            Some("") => return Err(ParseNaiveArnError::MissingService),
            Some(service) => service.into(),
        };

        let region = match elements.next() {
            None => return Err(ParseNaiveArnError::NotEnoughElements),
            Some("") => None,
            Some(region) => Some(region.into()),
        };

        let account_id = match elements.next() {
            None => return Err(ParseNaiveArnError::NotEnoughElements),
            Some("") => None,
            Some(account_id) => Some(account_id.into()),
        };

        let resource = match elements.next() {
            None => return Err(ParseNaiveArnError::NotEnoughElements),
            Some("") => return Err(ParseNaiveArnError::MissingResource),
            Some(resource) => resource.into(),
        };

        Ok(NaiveArn {
            partition,
            service,
            region,
            account_id,
            resource,
        })
    }
}

impl<'a> fmt::Display for NaiveArn<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "arn:{}:{}:{}:{}:{}",
            self.partition,
            self.service,
            self.region.unwrap_or_default(),
            self.account_id.unwrap_or_default(),
            self.resource
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseNaiveArnError {
    NotEnoughElements,
    MissingPrefix,
    MissingPartition,
    MissingService,
    MissingResource,
}

impl fmt::Display for ParseNaiveArnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseNaiveArnError::NotEnoughElements => write!(f, "Not enough elements"),
            ParseNaiveArnError::MissingPrefix => write!(f, "Missing 'arn:' prefix"),
            ParseNaiveArnError::MissingPartition => write!(f, "Missing partition element"),
            ParseNaiveArnError::MissingService => write!(f, "Missing service element"),
            ParseNaiveArnError::MissingResource => write!(f, "Missing resource element"),
        }
    }
}

impl error::Error for ParseNaiveArnError {}

#[cfg(test)]
mod tests {
    use super::{NaiveArn, ParseNaiveArnError};

    #[test]
    fn resource_type_with_slash() {
        let arn_str = "arn:aws:ec2:us-east-1:123456789012:vpc/vpc-fd580e98";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, String::from("aws"));
        assert_eq!(arn.service, String::from("ec2"));
        assert_eq!(arn.region, Some("us-east-1"));
        assert_eq!(arn.account_id, Some("123456789012"));
        assert_eq!(arn.resource, String::from("vpc/vpc-fd580e98"));

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn no_resource_type() {
        let arn_str = "arn:aws:codecommit:us-east-1:123456789012:MyDemoRepo";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "codecommit");
        assert_eq!(arn.region, Some("us-east-1"));
        assert_eq!(arn.account_id, Some("123456789012"));
        assert_eq!(arn.resource, "MyDemoRepo");

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn resource_type_with_multiple_colons() {
        let arn_str =
            "arn:aws:logs:us-east-1:123456789012:log-group:my-log-group*:log-stream:my-log-stream*";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "logs");
        assert_eq!(arn.region, Some("us-east-1"));
        assert_eq!(arn.account_id, Some("123456789012"));
        assert_eq!(
            arn.resource,
            "log-group:my-log-group*:log-stream:my-log-stream*"
        );

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn resource_type_with_colon() {
        let arn_str = "arn:aws:cloudwatch:us-east-1:123456789012:alarm:MyAlarmName";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "cloudwatch");
        assert_eq!(arn.region, Some("us-east-1"));
        assert_eq!(arn.account_id, Some("123456789012"));
        assert_eq!(arn.resource, "alarm:MyAlarmName");

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn resource_with_single_slash() {
        let arn_str =
            "arn:aws:kinesisvideo:us-east-1:123456789012:stream/example-stream-name/0123456789012";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "kinesisvideo");
        assert_eq!(arn.region, Some("us-east-1"));
        assert_eq!(arn.account_id, Some("123456789012"));
        assert_eq!(arn.resource, "stream/example-stream-name/0123456789012");

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn resource_with_multiple_slashes() {
        let arn_str =
            "arn:aws:macie:us-east-1:123456789012:trigger/example61b3df36bff1dafaf1aa304b0ef1a975/alert/example8780e9ca227f98dae37665c3fd22b585";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "macie");
        assert_eq!(arn.region, Some("us-east-1"));
        assert_eq!(arn.account_id, Some("123456789012"));
        assert_eq!(
            arn.resource,
            "trigger/example61b3df36bff1dafaf1aa304b0ef1a975/alert/example8780e9ca227f98dae37665c3fd22b585"
        );

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn no_region_no_account_id() {
        let arn_str = "arn:aws:s3:::my_corporate_bucket";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "s3");
        assert_eq!(arn.region, None);
        assert_eq!(arn.account_id, None);
        assert_eq!(arn.resource, "my_corporate_bucket");

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn spaces() {
        let arn_str = "arn:aws:artifact:::report-package/Certifications and Attestations/SOC/*";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "artifact");
        assert_eq!(arn.region, None);
        assert_eq!(arn.account_id, None);
        assert_eq!(
            arn.resource,
            "report-package/Certifications and Attestations/SOC/*"
        );

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn malformed_arn_no_arn_prefix() {
        let arn_str = "something:aws:s3:::my_corporate_bucket";
        let arn = NaiveArn::parse(arn_str);

        assert_eq!(arn, Err(ParseNaiveArnError::MissingPrefix))
    }

    #[test]
    fn malformed_arn_empty_string() {
        let arn_str = "";
        let arn = NaiveArn::parse(arn_str);

        assert_eq!(arn, Err(ParseNaiveArnError::MissingPrefix))
    }

    #[test]
    fn malformed_arn_just_prefix() {
        let arn_str = "arn:";
        let arn = NaiveArn::parse(arn_str);

        assert_eq!(arn, Err(ParseNaiveArnError::MissingPartition))
    }

    #[test]
    fn malformed_arn_not_enough_colons() {
        let arn_str = "arn:aws:a4b:us-east-1:123456789012";
        let arn = NaiveArn::parse(arn_str);

        assert_eq!(arn, Err(ParseNaiveArnError::NotEnoughElements))
    }

    #[test]
    fn malformed_arn_missing_partition() {
        let arn_str = "arn::ec2:us-east-1:123456789012:vpc/vpc-fd580e98";
        let arn = NaiveArn::parse(arn_str);

        assert_eq!(arn, Err(ParseNaiveArnError::MissingPartition))
    }

    #[test]
    fn malformed_arn_missing_service() {
        let arn_str = "arn:aws::us-east-1:123456789012:vpc/vpc-fd580e98";
        let arn = NaiveArn::parse(arn_str);

        assert_eq!(arn, Err(ParseNaiveArnError::MissingService))
    }

    #[test]
    fn malformed_arn_missing_resource() {
        let arn_str = "arn:aws:ec2:us-east-1:123456789012:";
        let arn = NaiveArn::parse(arn_str);

        assert_eq!(arn, Err(ParseNaiveArnError::MissingResource))
    }

    #[test]
    fn service_apigateway_resource_with_colon_and_slash() {
        let arn_str =
            "arn:aws:apigateway:us-east-1::a123456789012bc3de45678901f23a45:/test/mydemoresource/*";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "apigateway");
        assert_eq!(arn.region, Some("us-east-1"));
        assert_eq!(arn.account_id, None);
        assert_eq!(
            arn.resource,
            "a123456789012bc3de45678901f23a45:/test/mydemoresource/*"
        );

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn service_execute_api() {
        let arn_str = "arn:aws:execute-api:us-east-1:123456789012:8kjmp19d1h/*/*/*/*";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "execute-api");
        assert_eq!(arn.region, Some("us-east-1"));
        assert_eq!(arn.account_id, Some("123456789012"));
        assert_eq!(arn.resource, "8kjmp19d1h/*/*/*/*");

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn service_sns() {
        let arn_str = "arn:aws:sns:*:123456789012:my_corporate_topic";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "sns");
        assert_eq!(arn.region, Some("*"));
        assert_eq!(arn.account_id, Some("123456789012"));
        assert_eq!(arn.resource, "my_corporate_topic");

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn service_sns_resource_with_colon() {
        let arn_str = "arn:aws:sns:us-east-1:123456789012:my_corporate_topic:02034b43-fefa-4e07-a5eb-3be56f8c54ce";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "sns");
        assert_eq!(arn.region, Some("us-east-1"));
        assert_eq!(arn.account_id, Some("123456789012"));
        assert_eq!(
            arn.resource,
            "my_corporate_topic:02034b43-fefa-4e07-a5eb-3be56f8c54ce"
        );

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn service_s3() {
        let arn_str = "arn:aws:s3:::my_corporate_bucket/exampleobject.png";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "s3");
        assert_eq!(arn.region, None);
        assert_eq!(arn.account_id, None);
        assert_eq!(arn.resource, "my_corporate_bucket/exampleobject.png");

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn service_s3_resource_with_wildcard() {
        let arn_str = "arn:aws:s3:::my_corporate_bucket/*";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "s3");
        assert_eq!(arn.region, None);
        assert_eq!(arn.account_id, None);
        assert_eq!(arn.resource, "my_corporate_bucket/*");

        assert_eq!(arn.to_string(), arn_str);
    }

    #[test]
    fn service_s3_resource_with_wildcard_and_multiple_slashes() {
        let arn_str = "arn:aws:s3:::my_corporate_bucket/Development/*";
        let arn = NaiveArn::parse(arn_str).unwrap();

        assert_eq!(arn.partition, "aws");
        assert_eq!(arn.service, "s3");
        assert_eq!(arn.region, None);
        assert_eq!(arn.account_id, None);
        assert_eq!(arn.resource, "my_corporate_bucket/Development/*");

        assert_eq!(arn.to_string(), arn_str);
    }
}
